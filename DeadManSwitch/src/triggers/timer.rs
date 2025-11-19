use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{sleep, Duration};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::config::Config;
use crate::error::Result;
use super::{TriggerEvent, TriggerSender, TriggerSource};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum TimerCommand {
    #[command(description = "Heartbeat - Reset the timer")]
    Alive,
    
    #[command(description = "Check status - Show time until trigger")]
    Status,
}

pub struct HeartbeatTimer {
    config: Config,
    trigger_tx: TriggerSender,
}

impl HeartbeatTimer {
    pub fn new(config: Config, trigger_tx: TriggerSender) -> Self {
        Self { config, trigger_tx }
    }

    pub async fn start(self) -> Result<()> {
        if self.config.telegram_bot_token.contains("TOKEN") {
            log::error!("[!] Invalid Telegram token for timer");
            return Err(crate::error::DmsError::Config("Invalid token".into()));
        }

        let bot = Bot::new(&self.config.telegram_bot_token);
        
        if let Err(e) = bot.get_me().await {
            log::error!("[!] Telegram connection failed: {:?}", e);
            return Err(crate::error::DmsError::Config(format!("Connection failed: {}", e)));
        }

        let timeout_duration = self.config.telegram_heartbeat_timeout;
        let last_heartbeat = Arc::new(Mutex::new(Instant::now()));
        let last_chat_id: Arc<Mutex<Option<ChatId>>> = Arc::new(Mutex::new(None));  // ← Track chat ID
        
        log::warn!("[!] Heartbeat timer started: {} seconds timeout", timeout_duration);
        log::warn!("[!] Timer bot listening for /alive and /status");

        // Spawn countdown monitor
        let trigger_tx = self.trigger_tx.clone();
        let last_heartbeat_monitor = Arc::clone(&last_heartbeat);
        let last_chat_id_monitor = Arc::clone(&last_chat_id);
        let bot_monitor = bot.clone();
        
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                
                let elapsed = {
                    let last = last_heartbeat_monitor.lock().unwrap();
                    last.elapsed().as_secs()
                };

                let remaining = timeout_duration.saturating_sub(elapsed);
                
                if remaining == 60 || remaining == 30 || remaining == 10 {
                    log::warn!("[!] {} seconds until DMS trigger", remaining);
                }

                if elapsed >= timeout_duration {
                    log::error!("[!] Heartbeat timeout exceeded ({} seconds)", elapsed);
                    log::error!("[!] No heartbeat received - triggering DMS");
                    
                    let chat_id_opt = {
                        let mut guard = last_chat_id_monitor.lock().unwrap();
                        guard.take()
                    };
                    // Send alert to Telegram if we have a chat_id
                    if let Some(chat_id) = chat_id_opt {
                        let message = format!(
                            "🚨☠️ DEAD MAN SWITCH ACTIVATED! ☠️🚨\n\n\
                            ⚠️ Heartbeat timeout exceeded: {} seconds\n\
                            💀 System shutdown initiated\n\n\
                            This is an automated security response.",
                            elapsed
                        );

                        if let Err(e) = bot_monitor.send_message(chat_id, message).await {
                            log::error!("[!] Failed to send Telegram alert: {:?}", e);
                        }
                    }
                    
                    let _ = trigger_tx.send(TriggerEvent::new(TriggerSource::Timer));

                    break;
                }
            }
        });


        // Telegram bot handler
        let bot_clone = bot.clone();
        let last_heartbeat_bot = Arc::clone(&last_heartbeat);
        let last_chat_id_bot = Arc::clone(&last_chat_id);
        
        let handler = Update::filter_channel_post()
            .branch(
                dptree::entry()
                    .filter_command::<TimerCommand>()
                    .endpoint(move |cmd: TimerCommand, msg: Message| {
                        let heartbeat = Arc::clone(&last_heartbeat_bot);
                        let chat_id_store = Arc::clone(&last_chat_id_bot);
                        let bot = bot_clone.clone();
                        let timeout = timeout_duration;
                        
                        async move {
                            // Store chat_id for future alerts
                            {
                                let mut stored_id = chat_id_store.lock().unwrap();
                                *stored_id = Some(msg.chat.id);
                            }
                            
                            match cmd {
                                TimerCommand::Alive => {
                                    {
                                        let mut last = heartbeat.lock().unwrap();
                                        *last = Instant::now();
                                    }
                                    
                                    let elapsed = {
                                        let last = heartbeat.lock().unwrap();
                                        last.elapsed().as_secs()
                                    };
                                    let remaining = timeout.saturating_sub(elapsed);
                                    let hours = remaining / 3600;
                                    let minutes = (remaining % 3600) / 60;
                                    let secs = remaining % 60;
                                    
                                    let response = format!(
                                        "✅ Heartbeat received - timer reset!\n⏱️ Time until trigger: {}h {}m {}s",
                                        hours, minutes, secs
                                    );
                                    let _ = bot.send_message(msg.chat.id, response).await;
                                    log::info!("[+] Heartbeat received - {} seconds remaining", remaining);
                                }
                                TimerCommand::Status => {
                                    let elapsed = {
                                        let last = heartbeat.lock().unwrap();
                                        last.elapsed().as_secs()
                                    };
                                    let remaining = timeout.saturating_sub(elapsed);
                                    
                                    let hours_rem = remaining / 3600;
                                    let minutes_rem = (remaining % 3600) / 60;
                                    let secs_rem = remaining % 60;
                                    

                                    let response = format!(
                                        "⏱️ Time until trigger: {}h {}m {}s\n\
                                        💡 Send /alive to reset timer",
                                        hours_rem, minutes_rem, secs_rem
                                    );
                                    let _ = bot.send_message(msg.chat.id, response).await;
                                    log::info!("[+] Status check - {} seconds remaining", remaining);
                                }
                            }
                            respond(())
                        }
                    }),
            );

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
