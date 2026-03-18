use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_port: u16,
    pub poll_interval_secs: u64,
    pub db_path: PathBuf,
    pub zalo_reader_path: PathBuf,
    pub my_name: String,
    pub groq_api_key: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Haviz");
        std::fs::create_dir_all(&data_dir).ok();

        let agent_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        Self {
            http_port: env_or("HAVIZ_PORT", "9999").parse().unwrap_or(9999),
            poll_interval_secs: env_or("HAVIZ_POLL_INTERVAL", "3").parse().unwrap_or(3),
            db_path: data_dir.join("haviz.db"),
            zalo_reader_path: agent_dir.join("helpers").join("zalo_reader"),
            my_name: env_or("HAVIZ_MY_NAME", ""),
            groq_api_key: std::env::var("GROQ_API_KEY").ok(),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
