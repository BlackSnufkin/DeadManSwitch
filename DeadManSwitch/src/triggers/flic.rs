use flic_rust_client::*;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use crate::config::Config;
use crate::error::{DmsError, Result};
use super::{TriggerEvent, TriggerSender, TriggerSource};

pub struct FlicMonitor {
    config: Config,
    trigger_tx: TriggerSender,
}

impl FlicMonitor {
    pub fn new(config: Config, trigger_tx: TriggerSender) -> Self {
        Self { config, trigger_tx }
    }

    pub async fn start(self) -> Result<()> {
        let (info_tx, mut info_rx) = mpsc::channel(1);
        let trigger_tx = self.trigger_tx.clone();

        let handler = event_handler(move |event| {
            match event {
                Event::ButtonSingleOrDoubleClickOrHold { 
                    click_type: ClickType::ButtonHold, .. 
                } => {
                    log::warn!("[!] Flic trigger activated");
                    let _ = trigger_tx.send(TriggerEvent::new(TriggerSource::Flic));
                }
                Event::GetInfoResponse { bd_addr_of_verified_buttons, .. } => {
                    let _ = info_tx.try_send(bd_addr_of_verified_buttons.clone());
                }
                _ => {}
            }
        });

        let addr = format!("{}:{}", self.config.flic_ip, self.config.flic_port);
        let client = Arc::new(
            FlicClient::new(&addr).await?
                .register_event_handler(handler).await
        );

        let client_cmd = Arc::clone(&client);
        let cmd_task = tokio::spawn(async move {
            client_cmd.submit(Command::CreateScanWizard { scan_wizard_id: 1 }).await;
            sleep(Duration::from_secs(5)).await;
            client_cmd.submit(Command::GetInfo).await;

            if let Some(buttons) = info_rx.recv().await {
                for (idx, addr) in buttons.iter().enumerate() {
                    client_cmd.submit(Command::CreateConnectionChannel {
                        conn_id: (idx as u32) + 1,
                        bd_addr: addr.clone(),
                        latency_mode: LatencyMode::NormalLatency,
                        auto_disconnect_time: 511,
                    }).await;
                    log::info!("[+] Flic armed: {}", addr);
                }
            }
        });

        let client_listen = Arc::clone(&client);
        let listen_task = tokio::spawn(async move {
            client_listen.listen().await;
        });

        let _ = tokio::try_join!(cmd_task, listen_task)
            .map_err(|e| DmsError::Join(e.to_string()))?;
        
        client.stop().await;
        Ok(())
    }
}