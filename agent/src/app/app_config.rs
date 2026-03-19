/// Application startup configuration helpers.
/// Handles loading .env.local from the monorepo root before Config::load().

/// Find and load .env.local from the monorepo root (one level above agent/).
/// Silently skips if the file does not exist.
pub fn load_dotenv() {
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env.local");

    if !env_path.exists() {
        return;
    }

    let content = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            std::env::set_var(key.trim(), val.trim());
        }
    }
}
