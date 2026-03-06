use std::{env, error::Error, fmt, net::SocketAddr, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub public_addr: String,
    pub market_base_url: String,
    pub data_dir: PathBuf,
    pub version: f32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let bind_addr = env_or_default("SENDER_BIND_ADDR", "0.0.0.0:4000");
        let bind_addr = bind_addr
            .parse()
            .map_err(|_| ConfigError::new("invalid SENDER_BIND_ADDR"))?;

        let public_addr = env_or_default("SENDER_PUBLIC_ADDR", "http://127.0.0.1:4000");
        let market_base_url = env_or_default("MARKET_BASE_URL", "http://127.0.0.1:3000");
        let data_dir = PathBuf::from(env_or_default("SENDER_DATA_DIR", "data"));

        let version_str = env_or_default("SENDER_VERSION", "1.0");
        let version = version_str
            .parse()
            .map_err(|_| ConfigError::new("invalid SENDER_VERSION"))?;

        Ok(Self {
            bind_addr,
            public_addr,
            market_base_url,
            data_dir,
            version,
        })
    }
}

#[derive(Debug)]
struct ConfigError {
    message: &'static str,
}

impl ConfigError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ConfigError {}

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}
