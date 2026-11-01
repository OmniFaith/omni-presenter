//! mDNS/DNS-SD advertisement so phones, tablets and OBS on the same network
//! can find the control panel and web screens without typing an IP address.
use mdns_sd::{DaemonEvent, IfKind, Receiver, ServiceDaemon, ServiceInfo};
use std::time::Duration;

#[cfg(target_os = "linux")]
pub mod install;

pub const SERVICE_TYPE: &str = "_omnipresenter._tcp.local.";
pub const HOSTNAME: &str = "omni-presenter.local.";
pub const DISPLAY_HOSTNAME: &str = "omni-presenter.local";

pub struct Discovery {
    daemon: ServiceDaemon,
    fullname: String,
    port: u16,
    control: bool,
    chat: bool,
    hostname: String,
    events: Receiver<DaemonEvent>,
    pub error: Option<String>,
    #[cfg(target_os = "linux")]
    avahi_child: Option<std::process::Child>,
}

impl Discovery {
    pub fn start(
        port: u16,
        control: bool,
        chat: bool,
        tls_port: Option<u16>,
        screens: &[String],
    ) -> Result<Self, String> {
        let lan_ip = crate::services::web::local_address();
        let sys_host = system_hostname();
        let display_hostname = if sys_host.is_empty() || sys_host == "localhost" {
            DISPLAY_HOSTNAME.to_string()
        } else {
            format!("{sys_host}.local")
        };
        let mdns_hostname = format!("{display_hostname}.");

        let ip_to_use = if lan_ip == "127.0.0.1" || lan_ip.is_empty() {
            ""
        } else {
            &lan_ip
        };

        let info = service_info(ip_to_use, port, control, chat, tls_port, screens, &mdns_hostname)?;
        let fullname = info.get_fullname().to_owned();
        let daemon = ServiceDaemon::new().map_err(|error| error.to_string())?;
        let setup = || {
            daemon.disable_interface(IfKind::IPv6)?;
            let events = daemon.monitor()?;
            daemon.register(info)?;
            Ok::<_, mdns_sd::Error>(events)
        };
        let events = match setup() {
            Ok(events) => events,
            Err(error) => {
                let _ = daemon.shutdown();
                return Err(error.to_string());
            }
        };

        #[cfg(target_os = "linux")]
        let avahi_child = {
            let device = device_label();
            let instance = format!("Omni Presenter ({device})");
            std::process::Command::new("avahi-publish-service")
                .arg("-f")
                .arg(&instance)
                .arg("_omnipresenter._tcp")
                .arg(port.to_string())
                .arg("path=/control")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .ok()
        };

        Ok(Self {
            daemon,
            fullname,
            port,
            control,
            chat,
            hostname: display_hostname,
            events,
            error: None,
            #[cfg(target_os = "linux")]
            avahi_child,
        })
    }

    pub fn poll(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                DaemonEvent::Error(error) => self.error = Some(error.to_string()),
                DaemonEvent::Announce(..) => self.error = None,
                DaemonEvent::NameChange(change)
                    if change.original.eq_ignore_ascii_case(&self.hostname) =>
                {
                    self.hostname = change.new_name.trim_end_matches('.').to_owned();
                }
                _ => {}
            }
        }
    }

    pub fn control_url(&self) -> Option<String> {
        (self.control && self.error.is_none())
            .then(|| crate::services::web::http_url(&self.hostname, self.port, "/control"))
    }

    pub fn chat_url(&self) -> Option<String> {
        (self.chat && self.error.is_none())
            .then(|| crate::services::web::http_url(&self.hostname, self.port, "/chat"))
    }
}

impl Drop for Discovery {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(mut child) = self.avahi_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = self.daemon.unregister(&self.fullname);
        if let Ok(stopped) = self.daemon.shutdown() {
            let _ = stopped.recv_timeout(Duration::from_millis(300));
        }
    }
}

pub fn system_hostname() -> String {
    #[cfg(unix)]
    {
        let mut buf = [0u8; 256];
        let res = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if res == 0 {
            if let Ok(s) = std::ffi::CStr::from_bytes_until_nul(&buf) {
                if let Ok(name) = s.to_str() {
                    let clean: String = name
                        .trim()
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
                        .collect();
                    if !clean.is_empty() {
                        return clean.to_lowercase();
                    }
                }
            }
        }
    }
    let env_name = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| std::fs::read_to_string("/etc/hostname"))
        .unwrap_or_default();
    let clean: String = env_name
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();
    if clean.is_empty() {
        "omni-presenter".to_string()
    } else {
        clean.to_lowercase()
    }
}

fn service_info(
    ip: &str,
    port: u16,
    control: bool,
    chat: bool,
    tls_port: Option<u16>,
    screens: &[String],
    hostname: &str,
) -> Result<ServiceInfo, String> {
    let device = device_label();
    let instance = format!("Omni Presenter ({device})");
    let mut properties: Vec<(&str, String)> =
        vec![("version", env!("CARGO_PKG_VERSION").to_string())];
    properties.push(("device", device));
    if control {
        properties.push(("path", "/control".into()));
    }
    if chat {
        properties.push(("chat", "/chat".into()));
        if let Some(port) = tls_port {
            properties.push(("tls_port", port.to_string()));
        }
    }
    if !screens.is_empty() {
        properties.push(("screen_path", "/screen/".into()));
        let mut paths = String::new();
        for screen in screens.iter().take(4) {
            let path = format!("/screen/{screen}");
            if paths.len() + path.len() + 1 > 240 {
                continue;
            }
            if !paths.is_empty() {
                paths.push(',');
            }
            paths.push_str(&path);
        }
        if !paths.is_empty() {
            properties.push(("screens", paths));
        }
    }
    let info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance,
        hostname,
        ip,
        port,
        properties.as_slice(),
    )
    .map_err(|error| error.to_string())?;

    if ip.is_empty() {
        Ok(info.enable_addr_auto())
    } else {
        Ok(info)
    }
}

fn device_label() -> String {
    let name = std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .unwrap_or_default();
    let name: String = name
        .trim()
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .take(40)
        .collect();
    if name.trim().is_empty() {
        "omni-presenter".into()
    } else {
        name.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> ServiceInfo {
        service_info(
            "192.168.1.10",
            8080,
            true,
            true,
            Some(8443),
            &["tela-1".to_string(), "tela-2".to_string()],
            HOSTNAME,
        )
        .unwrap()
    }

    #[test]
    fn advertisement_publishes_host_control_chat_and_screens() {
        let info = info();
        assert_eq!(info.get_hostname(), HOSTNAME);
        assert_eq!(info.get_port(), 8080);
        assert_eq!(info.get_property_val_str("path"), Some("/control"));
        assert_eq!(info.get_property_val_str("chat"), Some("/chat"));
        assert_eq!(info.get_property_val_str("tls_port"), Some("8443"));
        assert_eq!(
            info.get_property_val_str("screens"),
            Some("/screen/tela-1,/screen/tela-2")
        );
        assert_eq!(
            info.get_property_val_str("version"),
            Some(env!("CARGO_PKG_VERSION"))
        );
        assert!(info.get_fullname().starts_with("Omni Presenter ("));
    }

    #[test]
    fn friendly_urls_omit_the_default_http_port() {
        assert_eq!(
            crate::services::web::http_url(DISPLAY_HOSTNAME, 80, "/control"),
            "http://omni-presenter.local/control"
        );
        assert_eq!(
            crate::services::web::http_url(DISPLAY_HOSTNAME, 3000, "/control"),
            "http://omni-presenter.local:3000/control"
        );
    }

    #[test]
    fn disabled_features_are_not_advertised() {
        let info = service_info("192.168.1.10", 9000, false, false, None, &[], HOSTNAME).unwrap();
        assert_eq!(info.get_property_val_str("path"), None);
        assert_eq!(info.get_property_val_str("chat"), None);
        assert_eq!(info.get_property_val_str("tls_port"), None);
        assert_eq!(info.get_property_val_str("screens"), None);
    }

    #[test]
    fn long_screen_names_do_not_prevent_discovery() {
        let info = service_info(
            "192.168.1.10",
            3000,
            true,
            true,
            None,
            &["a".repeat(300), "auditorio".into()],
            HOSTNAME,
        )
        .unwrap();
        assert_eq!(info.get_property_val_str("screen_path"), Some("/screen/"));
        assert_eq!(
            info.get_property_val_str("screens"),
            Some("/screen/auditorio")
        );
    }

    #[test]
    #[ignore = "requires a local network with multicast enabled"]
    fn live_discovery_resolves_address_and_custom_port() {
        use std::{
            io::{Read, Write},
            net::TcpStream,
        };

        let server = crate::services::web_http::HttpServer::bind("0.0.0.0", 0).unwrap();
        let port = server.port;
        let mut discovery =
            Discovery::start(port, true, true, None, &["auditorio".into()]).unwrap();
        let browser = ServiceDaemon::new().unwrap();
        browser.disable_interface(IfKind::IPv6).unwrap();
        let events = browser.browse(SERVICE_TYPE).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut resolved = None;
        while std::time::Instant::now() < deadline {
            discovery.poll();
            if let Ok(mdns_sd::ServiceEvent::ServiceResolved(service)) =
                events.recv_timeout(Duration::from_millis(250))
                && service.get_port() == port
                && service.get_property_val_str("path") == Some("/control")
            {
                resolved = Some(service);
                break;
            }
        }
        let _ = browser.stop_browse(SERVICE_TYPE);
        if let Ok(stopped) = browser.shutdown() {
            let _ = stopped.recv_timeout(Duration::from_millis(300));
        }
        let service = resolved.expect("the service was not resolved over the local network");
        assert!(!service.get_addresses_v4().is_empty());
        assert_eq!(service.get_property_val_str("chat"), Some("/chat"));
        assert_eq!(
            service.get_property_val_str("screens"),
            Some("/screen/auditorio")
        );
        assert!(
            discovery
                .control_url()
                .unwrap()
                .ends_with(&format!(":{port}/control"))
        );

        let worker = std::thread::spawn(move || {
            let request = server
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                .unwrap();
            assert_eq!(request.url(), "/control");
            request
                .respond(tiny_http::Response::from_string("discovered control panel"))
                .unwrap();
        });
        // The client connects using the address and port received from discovery.
        let address = *service.get_addresses_v4().iter().next().unwrap();
        let mut socket = TcpStream::connect((address, service.get_port())).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        socket
            .write_all(b"GET /control HTTP/1.0\r\nHost: omni-presenter.local\r\n\r\n")
            .unwrap();
        let mut response = String::new();
        socket.read_to_string(&mut response).unwrap();
        assert!(response.ends_with("discovered control panel"));
        worker.join().unwrap();
    }
}
