use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub const DEFAULT_PASSWORD: &str = "omni-presenter";

pub fn hash(password: &str) -> Result<String, String> {
    if password.is_empty() || password.len() > 1024 {
        return Err("A senha deve ter entre 1 e 1024 bytes.".into());
    }
    let mut salt = [0; 16];
    getrandom::fill(&mut salt).map_err(|e| e.to_string())?;
    Argon2::default()
        .hash_password_with_salt(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify(hash: &str, password: &str) -> bool {
    if password.len() > 1024 {
        return false;
    }
    // Older profiles have no configured password until edited in Settings.
    if hash.is_empty() {
        return password == DEFAULT_PASSWORD;
    }
    PasswordHash::new(hash).is_ok_and(|hash| {
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    })
}
