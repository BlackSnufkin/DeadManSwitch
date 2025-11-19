use rusb::{Context, UsbContext};
use std::collections::HashSet;
use tokio::task;
use crate::config::Config;
use crate::error::Result;
use super::{TriggerEvent, TriggerSender, TriggerSource};

#[derive(PartialEq, Eq, Hash, Clone)]
struct DeviceId {
    vendor: u16,
    product: u16,
}

pub struct UsbMonitor {
    config: Config,
    trigger_tx: TriggerSender,
}

impl UsbMonitor {
    pub fn new(config: Config, trigger_tx: TriggerSender) -> Self {
        Self { config, trigger_tx }
    }

    pub async fn start(self) -> Result<()> {
        task::spawn_blocking(move || self.run()).await
            .map_err(|_| rusb::Error::Other)?
    }

    fn run(self) -> Result<()> {
        let ctx = Context::new()?;
        let mut known = HashSet::new();
        
        log::warn!("[!] USB trigger: {:04x}:{:04x}", 
                   self.config.usb_vendor_id, self.config.usb_product_id);

        loop {
            for device in ctx.devices()?.iter() {
                if let Ok(desc) = device.device_descriptor() {
                    let id = DeviceId {
                        vendor: desc.vendor_id(),
                        product: desc.product_id(),
                    };
                    
                    if known.insert(id.clone()) {
                        if id.vendor == self.config.usb_vendor_id 
                            && id.product == self.config.usb_product_id {
                            log::warn!("[!] USB trigger activated");
                            let _ = self.trigger_tx.send(TriggerEvent::new(TriggerSource::Usb));
                            return Ok(());
                        }
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));  // ← CHANGED: 100ms for faster detection
        }
    }
}