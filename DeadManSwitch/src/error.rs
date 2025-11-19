use thiserror::Error;

#[derive(Error, Debug)]
pub enum DmsError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    #[error("USB error: {0}")]
    Usb(#[from] rusb::Error),
    
    #[allow(dead_code)]
    #[error("Telegram error: {0}")]
    Telegram(String),
    
    #[error("Flic error: {0}")]
    Flic(#[from] anyhow::Error),
    
    #[error("Task join error: {0}")]
    Join(String),
}

pub type Result<T> = std::result::Result<T, DmsError>;