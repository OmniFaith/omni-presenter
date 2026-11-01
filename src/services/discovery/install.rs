//! Linux installer integration for browser resolution of `.local` hostnames.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process::Command,
};

/// Called by the installer, before launching the desktop application.
pub fn configure() -> Result<(), String> {
    if unsafe { libc::geteuid() } != 0 {
        return Err(
            crate::core::i18n::Language::default()
                .text("services-mdns-setup-must-be-run-by-the-installer-as-an")
                .to_string(),
        );
    }
    let module = unsafe {
        libc::dlopen(
            c"libnss_mdns4_minimal.so.2".as_ptr(),
            libc::RTLD_LAZY | libc::RTLD_LOCAL,
        )
    };
    if module.is_null() {
        return Err(
            crate::core::i18n::Language::default()
                .text("services-the-mdns-nss-module-is-not-installed-run-the-installer")
                .to_string(),
        );
    }
    unsafe {
        libc::dlclose(module);
    }
    // Installing only the responder does not connect Firefox/getaddrinfo to mDNS.
    // The installer also installs the nss-mdns package before invoking this command.
    let status = Command::new("systemctl")
        .args(["enable", "--now", "avahi-daemon.service"])
        .status()
        .map_err(|error| {
            crate::core::i18n::Language::default().format(
                "services-could-not-start-avahi",
                &[("error", &error.to_string())],
            )
        })?;
    if !status.success() {
        return Err(
            crate::core::i18n::Language::default()
                .text("services-could-not-enable-the-system-mdns-service")
                .to_string(),
        );
    }
    configure_nss_file(Path::new("/etc/nsswitch.conf"))?;
    println!(
        "{}",
        crate::core::i18n::Language::default()
            .text("services-mdns-configured-avahi-and-local-name-resolution-are-active")
    );
    Ok(())
}

fn configure_nss(source: &str) -> Result<String, String> {
    let mut result = String::with_capacity(source.len() + 40);
    let mut found = false;
    for line in source.split_inclusive('\n') {
        let Some((prefix, body)) = line.split_once(':') else {
            result.push_str(line);
            continue;
        };
        if prefix.trim() != "hosts" {
            result.push_str(line);
            continue;
        }
        if found {
            return Err(
                crate::core::i18n::Language::default()
                    .text("services-the-nss-configuration-contains-more-than-one-hosts-line")
                    .to_string(),
            );
        }
        found = true;
        let (body, comment) = body.split_once('#').unwrap_or((body, ""));
        let mut tokens: Vec<&str> = body.split_whitespace().collect();
        let resolver = tokens
            .iter()
            .position(|token| matches!(*token, "resolve" | "dns"))
            .unwrap_or(tokens.len());
        let mdns = tokens.iter().position(|token| {
            matches!(*token, "mdns" | "mdns4" | "mdns_minimal" | "mdns4_minimal")
        });
        if mdns.is_some_and(|index| index < resolver) {
            result.push_str(line);
            continue;
        }
        tokens.splice(resolver..resolver, ["mdns4_minimal", "[NOTFOUND=return]"]);
        result.push_str(prefix);
        result.push_str(": ");
        result.push_str(&tokens.join(" "));
        if !comment.is_empty() {
            result.push_str(" #");
            result.push_str(comment.trim_end_matches('\n'));
        }
        if line.ends_with('\n') {
            result.push('\n');
        }
    }
    if !found {
        return Err(
            crate::core::i18n::Language::default()
                .text("services-the-nss-configuration-has-no-hosts-line")
                .to_string(),
        );
    }
    Ok(result)
}

fn configure_nss_file(path: &Path) -> Result<(), String> {
    let update = || -> std::io::Result<()> {
        let path = path.canonicalize()?;
        let source = fs::read_to_string(&path)?;
        let configured = configure_nss(&source).map_err(std::io::Error::other)?;
        if configured == source {
            return Ok(());
        }
        let metadata = fs::metadata(&path)?;
        let backup = path.with_extension("conf.omni-presenter-backup");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)
        {
            Ok(mut file) => {
                file.set_permissions(metadata.permissions())?;
                file.write_all(source.as_bytes())?;
                file.sync_all()?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        let temporary = path.with_extension(format!("conf.omni-presenter-{}", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            file.set_permissions(fs::Permissions::from_mode(metadata.mode()))?;
            file.write_all(configured.as_bytes())?;
            file.sync_all()?;
            std::os::unix::fs::chown(&temporary, Some(metadata.uid()), Some(metadata.gid()))?;
            fs::rename(&temporary, &path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    };
    update().map_err(|error| {
        crate::core::i18n::Language::default().format(
            "services-could-not-integrate-the-mdns-resolver",
            &[("error", &error.to_string())],
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_mdns_before_dns_and_resolved_preserving_other_databases() {
        for hosts in [
            "files dns",
            "mymachines resolve [!UNAVAIL=return] files myhostname dns",
            "files [SUCCESS=return] dns mdns4",
        ] {
            let source = format!("passwd: files systemd\nhosts: {hosts} # rede\nnetworks: files\n");
            let updated = configure_nss(&source).unwrap();
            assert!(updated.starts_with("passwd: files systemd\n"));
            assert!(updated.ends_with(" # rede\nnetworks: files\n"));
            let position = updated.find("mdns4_minimal [NOTFOUND=return]").unwrap();
            assert!(position < updated.find("dns ").unwrap());
            if let Some(resolved) = updated.find("resolve ") {
                assert!(position < resolved);
            }
            assert_eq!(configure_nss(&updated).unwrap(), updated);
        }
    }

    #[test]
    fn preserves_existing_mdns_and_rejects_missing_or_ambiguous_hosts() {
        let source = "hosts: files mdns_minimal [NOTFOUND=return] dns\n";
        assert_eq!(configure_nss(source).unwrap(), source);
        assert!(configure_nss("passwd: files\n").is_err());
        assert!(configure_nss("hosts: files dns\nhosts: dns\n").is_err());
    }

    #[test]
    fn installation_is_atomic_idempotent_and_keeps_the_original_backup() {
        let folder = std::env::temp_dir().join(crate::core::preferences::new_id("omni-presenter-nss"));
        fs::create_dir(&folder).unwrap();
        let path = folder.join("nsswitch.conf");
        let original = "passwd: files\nhosts: files dns\n";
        fs::write(&path, original).unwrap();
        configure_nss_file(&path).unwrap();
        let updated = fs::read_to_string(&path).unwrap();
        configure_nss_file(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), updated);
        assert_eq!(
            fs::read_to_string(path.with_extension("conf.omni-presenter-backup")).unwrap(),
            original
        );
        fs::remove_dir_all(folder).unwrap();
    }
}
