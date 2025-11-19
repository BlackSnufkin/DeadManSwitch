#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;
mod triggers;
mod actions;
mod ui;

use clap::Parser;
use simplelog::*;
use std::process;
use std::sync::{Arc, Mutex};
use crate::error::Result;
use crate::triggers::*;
use crate::actions::ActionExecutor;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "all")]
    mode: String,
    
    #[clap(short, long)]
    trigger: bool,
}

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info, 
            simplelog::Config::default(),
            TerminalMode::Mixed, 
            ColorChoice::Auto
        )
    ]).unwrap();

    let args = Args::parse();
    let config = config::Config::default()?;

    if args.trigger {
        log::warn!("[!] Manual trigger mode");
        let executor = ActionExecutor::new(config.clone());
        executor.execute();
        ui::show_alert();
        return Ok(());
    }

    // Flag to signal when to show UI
    let should_show_ui = Arc::new(Mutex::new(false));
    let should_show_ui_clone = Arc::clone(&should_show_ui);

    // Spawn async monitors in background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_monitors(config, args.mode, should_show_ui_clone).await.ok();
        });
    });

    // Main thread waits for trigger signal, then shows UI
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        let triggered = *should_show_ui.lock().unwrap();
        if triggered {
            ui::show_alert();  // ← UI on main thread
            process::exit(0);
        }
    }
}

async fn run_monitors(
    config: config::Config, 
    mode: String,
    should_show_ui: Arc<Mutex<bool>>
) -> Result<()> {
    let executor = ActionExecutor::new(config.clone());
    let modes: Vec<_> = mode.split(',').map(str::trim).collect();
    let run_all = modes.contains(&"all");
    
    let (tx, mut rx) = triggers::create_trigger_channel();
    let mut tasks = vec![];
    let mut active_modes = vec![];

    if run_all || modes.contains(&"timer") {
        let timer = timer::HeartbeatTimer::new(config.clone(), tx.clone());
        let timer_task = tokio::spawn(async move { timer.start().await });
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if !timer_task.is_finished() {
            tasks.push(timer_task);
            active_modes.push("timer");
        } else {
            log::warn!("[!] Heartbeat timer skipped");
        }
    }

    if run_all || modes.contains(&"net") {
        let listener = network::NetworkListener::new(config.clone(), tx.clone());
        tasks.push(tokio::spawn(async move { listener.start().await }));
        active_modes.push("net");
    }

    if run_all || modes.contains(&"bot") {
        let listener = telegram::TelegramListener::new(config.clone(), tx.clone());
        let bot_task = tokio::spawn(async move { listener.start().await });
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        if !bot_task.is_finished() {
            tasks.push(bot_task);
            active_modes.push("bot");
        } else {
            log::warn!("[!] Telegram skipped");
        }
    }

    if run_all || modes.contains(&"usb") {
        let monitor = usb::UsbMonitor::new(config.clone(), tx.clone());
        tasks.push(tokio::spawn(async move { monitor.start().await }));
        active_modes.push("usb");
    }

    if run_all || modes.contains(&"flic") {
        let monitor = flic::FlicMonitor::new(config.clone(), tx.clone());
        tasks.push(tokio::spawn(async move { monitor.start().await }));
        active_modes.push("flic");
    }

    if active_modes.is_empty() {
        log::error!("[!] No valid modes");
        return Ok(());
    }

    log::info!("[+] DMS armed: {:?}", active_modes);
    ActionExecutor::send_notification(&active_modes);

    if let Some(event) = rx.recv().await {
        log::warn!("[!] Trigger from {:?}", event.source);
        executor.execute();
        *should_show_ui.lock().unwrap() = true;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        process::exit(0);
    }

    Ok(())
}