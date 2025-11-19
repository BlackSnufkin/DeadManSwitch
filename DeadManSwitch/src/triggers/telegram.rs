use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::config::Config;
use crate::error::Result;
use super::{TriggerEvent, TriggerSender, TriggerSource};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Manual trigger - Activate DMS immediately")]
    Dms(String),
}

pub struct TelegramListener {
    config: Config,
    trigger_tx: TriggerSender,
}

impl TelegramListener {
    pub fn new(config: Config, trigger_tx: TriggerSender) -> Self {
        Self { config, trigger_tx }
    }

    pub async fn start(self) -> Result<()> {
        if self.config.telegram_bot_token.contains("TOKEN") {
            log::error!("[!] Invalid Telegram token");
            return Err(crate::error::DmsError::Config("Invalid token".into()));
        }
        
        let bot = Bot::new(&self.config.telegram_bot_token);
        
        if let Err(e) = bot.get_me().await {
            log::error!("[!] Telegram connection failed: {:?}", e);
            return Err(crate::error::DmsError::Config(format!("Connection failed: {}", e)));
        }
        
        log::warn!("[!] Telegram manual trigger active");

        let trigger_tx = self.trigger_tx.clone();
        let expected_cmd = self.config.telegram_command.clone();
        let bot_clone = bot.clone();

        let handler = Update::filter_channel_post()
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(move |cmd: Command, msg: Message| {
                        let tx = trigger_tx.clone();
                        let expected = expected_cmd.clone();
                        let bot = bot_clone.clone();
                        
                        async move {
                            let Command::Dms(param) = cmd;
                            if param == expected {
                                log::warn!("[!] Manual Telegram trigger activated");
                                let _ = bot.send_message(msg.chat.id, "🚨☠️ Dead Man Switch ACTIVATED! 🚨☠️").await;
                                let _ = tx.send(TriggerEvent::new(TriggerSource::Telegram));
                            } else {
                                let _ = bot.send_message(msg.chat.id, "❌ Invalid command parameter").await;
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