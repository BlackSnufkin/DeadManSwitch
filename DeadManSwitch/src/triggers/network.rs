use std::net::{SocketAddr, UdpSocket};
use tokio::task;
use crate::config::Config;
use crate::error::Result;
use super::{TriggerEvent, TriggerSender, TriggerSource};

pub struct NetworkListener {
    config: Config,
    trigger_tx: TriggerSender,
}

impl NetworkListener {
    pub fn new(config: Config, trigger_tx: TriggerSender) -> Self {
        Self { config, trigger_tx }
    }

    pub async fn start(self) -> Result<()> {
        task::spawn_blocking(move || self.run()).await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
    }

    fn run(self) -> Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.broadcast_port).parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(false)?;
        
        log::warn!("[!] Network trigger: {}", addr);
        
        let mut buf = vec![0u8; 4096];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, _)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    if msg.eq_ignore_ascii_case(&self.config.broadcast_message) {
                        log::warn!("[!] Network trigger activated");
                        let _ = self.trigger_tx.send(TriggerEvent::new(TriggerSource::Network));
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    pub fn send_trigger_broadcast(config: &Config) -> Result<()> {
        let addr: SocketAddr = format!("255.255.255.255:{}", config.broadcast_port).parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.send_to(config.broadcast_message.as_bytes(), addr)?;
        
        log::warn!("[!] Broadcast trigger sent");
        Ok(())
    }
}