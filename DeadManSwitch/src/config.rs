use local_ip_address::local_ip;
use regex::Regex;
use std::net::Ipv4Addr;
use crate::error::{DmsError, Result};

lazy_static::lazy_static! {
    static ref IPV4_REGEX: Regex = 
        Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
}

#[derive(Clone, Debug)]
pub struct Config {
    pub telegram_bot_token: String,
    pub telegram_heartbeat_timeout: u64,  // ← NEW: seconds without heartbeat before trigger
    pub broadcast_port: u16,
    pub broadcast_message: String,
    pub telegram_command: String,
    pub usb_vendor_id: u16,
    pub usb_product_id: u16,
    pub flic_ip: String,
    pub flic_port: u16,
}

impl Config {
    pub fn new(
        telegram_bot_token: String,
        telegram_heartbeat_timeout: u64,
        broadcast_port: u16,
        broadcast_message: String,
        telegram_command: String,
        usb_vendor_id: u16,
        usb_product_id: u16,
        mut flic_ip: String,
        flic_port: u16,
    ) -> Result<Self> {
        if !IPV4_REGEX.is_match(&flic_ip) {
            flic_ip = Self::auto_detect_flic_ip()?;
            log::warn!("[!] Flic IP auto-detected: {}", flic_ip);
        }
        
        Ok(Self {
            telegram_bot_token,
            telegram_heartbeat_timeout,
            broadcast_port,
            broadcast_message,
            telegram_command,
            usb_vendor_id,
            usb_product_id,
            flic_ip,
            flic_port,
        })
    }

    fn auto_detect_flic_ip() -> Result<String> {
        let local_ip = local_ip()
            .map_err(|e| DmsError::Config(format!("Failed to get local IP: {}", e)))?;
        
        match local_ip {
            std::net::IpAddr::V4(ipv4) => {
                let mut octets = ipv4.octets();
                octets[3] = 242;
                Ok(Ipv4Addr::from(octets).to_string())
            }
            std::net::IpAddr::V6(_) => {
                Err(DmsError::Config("IPv6 not supported for Flic IP".into()))
            }
        }
    }

    pub fn default() -> Result<Self> {
        Self::new(
            "TELEGRAM_BOT_TOKEN".to_string(),
            30,  // ← 1 hour timeout by default
            45370,
            "trigger_dms".to_string(),
            "execute".to_string(),
            0x090c,
            0x1000,
            "auto".to_string(),
            5551,
        )
    }
}