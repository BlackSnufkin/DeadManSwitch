pub mod network;
pub mod telegram;
pub mod usb;
pub mod flic;
pub mod timer;  // ← NEW

use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSource {
    Network,
    Telegram,
    Usb,
    Flic,
    Timer,
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub source: TriggerSource,
    #[allow(dead_code)]
    pub timestamp: std::time::Instant,
}

impl TriggerEvent {
    pub fn new(source: TriggerSource) -> Self {
        Self {
            source,
            timestamp: std::time::Instant::now(),
        }
    }
}

pub type TriggerSender = mpsc::UnboundedSender<TriggerEvent>;
pub type TriggerReceiver = mpsc::UnboundedReceiver<TriggerEvent>;

pub fn create_trigger_channel() -> (TriggerSender, TriggerReceiver) {
    mpsc::unbounded_channel()
}