//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Uncomment above line when you want to hide the console window
use anyhow::Result;
use clap::{Parser};
use flic_rust_client::{ClickType, ConnectionStatus, Event, Command as FlicCommand, event_handler, FlicClient, LatencyMode};
use iced::alignment::{Horizontal, Vertical};
use iced::window;
use iced::{executor, Application, Column, Command as IcedCommand, Container, Element, Length, Settings, Text};
use log::{error, info, warn, LevelFilter};
use notify_rust::Notification;
use rusb::{Context, Device, UsbContext, Error};
use simplelog::{ColorChoice, CombinedLogger, Config as LogConfig, TermLogger, TerminalMode};
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::process::Command as OsCommand;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Instant};
use teloxide::prelude::*;
use teloxide::types::Message as BotMessage;
use teloxide::utils::command::BotCommands;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

struct DeadManSwitchConfig {
    telegram_bot_token: String,
    broadcast_port: u16,
    broadcast_message: String,
    telegram_command: String,
    usb_vendor_id: u16,
    usb_product_id: u16,
    flic_ip: String,
    flic_port: u16,
}

impl DeadManSwitchConfig {
    fn new(
        telegram_bot_token: String,
        broadcast_port: u16,
        broadcast_message: String,
        telegram_command: String,
        usb_vendor_id: u16,
        usb_product_id: u16,
        flic_ip: String,
        flic_port: u16,
    ) -> Self {
        Self {
            telegram_bot_token,
            broadcast_port,
            broadcast_message,
            telegram_command,
            usb_vendor_id,
            usb_product_id,
            flic_ip,
            flic_port,
        }
    }
}


lazy_static::lazy_static! {
    static ref CONFIG: DeadManSwitchConfig = DeadManSwitchConfig::new(
        "<TELEGRAM BOT API TOKEN>".to_string(),
        45370,
        "trigger_dms".to_string(),
        "execute".to_string(),
        0x090c,
        0x1000,
        "<FLIC BUTTON SERVER>".to_string(), 
        5551, 
    );
}

#[derive(Clone)]
enum MessageSource {
    Network,
    Telegram,
}

#[derive(Clone)]
struct TriggerMessage {
    source: MessageSource,
    content: String,
}

struct BlackBackgroundStyle;

impl iced::container::StyleSheet for BlackBackgroundStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: iced::Color::BLACK.into(),
            ..Default::default()
        }
    }
}


struct DeadManSwitchApp {
    remaining_seconds: i32,
    flash_emergency: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    FlashEmergency,
}

impl Application for DeadManSwitchApp {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (DeadManSwitchApp, IcedCommand<Message>) {
        (
            DeadManSwitchApp {
                remaining_seconds: 3,
                flash_emergency: true,
            },
            IcedCommand::batch(vec![
                IcedCommand::perform(Self::wait_a_second(), Message::Tick),
                IcedCommand::perform(Self::wait_half_second(), |_| Message::FlashEmergency),
            ]),
        )
    }

    fn title(&self) -> String {
        String::from("EMERGENCY ALERT: Dead Man Switch Activated")
    }

    fn update(&mut self, message: Message) -> IcedCommand<Message> {
        match message {
            Message::Tick(_) => {
                if self.remaining_seconds > 0 {
                    self.remaining_seconds -= 1;
                }
                IcedCommand::perform(Self::wait_a_second(), Message::Tick)
            }
            Message::FlashEmergency => {
                self.flash_emergency = !self.flash_emergency;
                IcedCommand::perform(Self::wait_half_second(), |_| Message::FlashEmergency)
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let emergency_text = Text::new("!!! Dead Man's Switch Activation !!!")
            .size(100)
            .color(if self.flash_emergency {
                iced::Color::from_rgb(1.0, 1.0, 1.0)
            } else {
                iced::Color::from_rgb(1.0, 0.0, 0.0)
            })
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let message = "**CRITICAL SECURITY ALERT: Dead Man's Switch Activation**
                    We have detected the activation of the Dead Man's Switch,.
                    This is an immediate and serious security threat.
                    IMMEDIATE ACTION REQUIRED:
                    - For security purposes, your computer will SHUTDOWN in 3 seconds.
                    - All VeraCrypt volumes will be dismounted to secure confidential data.
                    If this is a false alarm, contact your system administrator IMMEDIATELY.";

        let message_text = Text::new(message)
            .size(55)
            .color([1.0, 1.0, 1.0])
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let countdown_label = Text::new(format!("Locking in: {} seconds", self.remaining_seconds))
            .size(60)
            .color([1.0, 0.0, 0.0])
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Bottom);

        Container::new(
            Column::new()
                .push(emergency_text)
                .push(message_text)
                .push(countdown_label)
                .spacing(20)
                .padding(60),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .style(BlackBackgroundStyle)
        .into()
    }
}


impl DeadManSwitchApp {
    async fn wait_a_second() -> Instant {
        let duration = Duration::from_secs(1);
        std::thread::sleep(duration);
        Instant::now()
    }

    async fn wait_half_second() -> Instant {
        let duration = Duration::from_millis(500);
        std::thread::sleep(duration);
        Instant::now()
    }

    fn get_screen_size() -> (i32, i32) {
        #[cfg(target_os = "windows")]
        {
            extern crate winapi;
            unsafe {
                let screen_dc = winapi::um::winuser::GetDC(std::ptr::null_mut());
                let width = winapi::um::wingdi::GetDeviceCaps(screen_dc, winapi::um::wingdi::HORZRES);
                let height = winapi::um::wingdi::GetDeviceCaps(screen_dc, winapi::um::wingdi::VERTRES);
                (width, height)
            }
        }
        
        #[cfg(target_os = "linux")]
        #[link(name = "X11")]
        extern "C" {}
        #[cfg(target_os = "linux")]
        {
            use x11::xlib;
            unsafe {
                let display = xlib::XOpenDisplay(std::ptr::null_mut());
                let screen_num = xlib::XDefaultScreen(display);
                let width = xlib::XDisplayWidth(display, screen_num);
                let height = xlib::XDisplayHeight(display, screen_num);
                xlib::XCloseDisplay(display);
                (width as i32, height as i32)
            }
        }
        #[cfg(target_os = "macos")]
        {
            extern crate cocoa;
            extern crate core_graphics;
            use cocoa::base::nil;
            use cocoa::appkit::NSScreen;
            use core_graphics::display::CGDisplay;
            use objc::runtime::YES;
            unsafe {
                let screen = NSScreen::mainScreen(nil);
                if screen != nil {
                    let rect = NSScreen::frame(screen);
                    return (rect.size.width as i32, rect.size.height as i32);
                }
            }
            (0, 0)
        }
    }

    fn run_alert(&self) {
        let size = Self::get_screen_size();
        let settings = Settings {
            window: window::Settings {
                size: (size.0 as u32, size.1 as u32),
                resizable: false,
                decorations: false,
                always_on_top: true,
                transparent: false,
                min_size: None,
                max_size: None,
                position: iced::window::Position::Centered,
                icon: None,
            },
            ..Default::default()
        };
        DeadManSwitchApp::run(settings).unwrap();
    }
}

struct TelegramBot {
    bot_token: String,
    latest_message: Arc<Mutex<Option<TriggerMessage>>>,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Activate the Dead Man Switch with run parameter.")]
    Dms(String),
}

impl TelegramBot {
    fn new(bot_token: String, latest_message: Arc<Mutex<Option<TriggerMessage>>>, command: String) -> Self {
        Self { bot_token, latest_message }
    }

    async fn start(&self) {
        let bot = Bot::new(self.bot_token.clone());
        info!("[+] Telegram Bot Trigger is ON");

        let channel_latest_message = Arc::clone(&self.latest_message);
        let bot_clone = bot.clone();
        let channel_post_handler = Update::filter_channel_post().branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(move |cmd: Command, msg: BotMessage | {
                    let latest_message = Arc::clone(&channel_latest_message);
                    let bot = bot_clone.clone();
                    async move {
                        if let Command::Dms(param) = cmd {
                            if param == CONFIG.telegram_command {
                                {
                                    let mut message = latest_message.lock().unwrap();
                                    *message = Some(TriggerMessage {
                                        source: MessageSource::Telegram,
                                        content: CONFIG.telegram_command.clone(),
                                    });
                                }

                                let response_text = "Dead Man Switch activated! 🚨☠️";
                                bot.send_message(msg.chat.id, response_text).await?;
                            }
                        }
                        respond(())
                    }
                }),
        );

        Dispatcher::builder(bot, channel_post_handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }
}

struct NetworkListener {
    broadcast_port: u16,
    broadcast_message: String,
    latest_message: Arc<Mutex<Option<TriggerMessage>>>,
}

impl NetworkListener {
    fn new(broadcast_port: u16, broadcast_message: String, latest_message: Arc<Mutex<Option<TriggerMessage>>>) -> Self {
        Self { broadcast_port, broadcast_message, latest_message }
    }

    fn listen(&self) {
        let server_address: SocketAddr = format!("0.0.0.0:{}", self.broadcast_port).parse().unwrap();
        let socket = UdpSocket::bind(server_address).expect("Failed to bind socket");
        info!("[+] Network Trigger is ON: {}", server_address);

        let mut buffer = [0; 4096];
        loop {
            let (size, _) = socket.recv_from(&mut buffer).expect("Failed to receive data");
            let message = String::from_utf8_lossy(&buffer[..size]).to_string();
            if message.to_lowercase() == self.broadcast_message.to_lowercase() {
                let mut msg = self.latest_message.lock().unwrap();
                *msg = Some(TriggerMessage {
                    source: MessageSource::Network,
                    content: message,
                });
                break;
            }
            buffer = [0; 4096];
        }
    }
}

struct UsbMonitor {
    usb_vendor_id: u16,
    usb_product_id: u16,
}

impl UsbMonitor {
    fn new(usb_vendor_id: u16, usb_product_id: u16) -> Self {
        Self { usb_vendor_id, usb_product_id }
    }

    fn monitor(&self, trigger_channel: std::sync::mpsc::Sender<()>) {
        let context = Context::new().expect("Failed to create USB context");
        let mut known_devices: HashSet<DeviceInfo> = HashSet::new();

        info!("[+] USB Device Trigger is ON");
        loop {
            let devices = context.devices().expect("Failed to get USB devices");
            for device in devices.iter() {
                let device_info = DeviceInfo::from_device(&device).expect("Failed to get device info");
                if known_devices.insert(device_info.clone()) {
                    if self.usb_trigger(&device).expect("Failed to check dead man switch") {
                        let _ = trigger_channel.send(());
                        return;
                    }
                }
            }
        }
    }

    fn usb_trigger<T: UsbContext>(&self, device: &Device<T>) -> Result<bool, Error> {
        let device_desc = device.device_descriptor()?;
        Ok(device_desc.vendor_id() == self.usb_vendor_id && device_desc.product_id() == self.usb_product_id)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct DeviceInfo {
    vendor_id: u16,
    product_id: u16,

}

impl DeviceInfo {
    fn from_device<T: UsbContext>(device: &Device<T>) -> Result<Self, Error> {
        let device_desc = device.device_descriptor()?;
        Ok(Self {
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),

        })
    }
}

struct FlicButton {
    ip: String,
    port: u16,
}

impl FlicButton {
    fn new(ip: String, port: u16) -> Self {
        Self { ip, port }
    }

    async fn handle(&self, trigger_channel: std::sync::mpsc::Sender<()>) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        let event = event_handler(move |event| {
            match event {
                Event::ConnectionStatusChanged { conn_id: _, connection_status: ConnectionStatus::Ready, disconnect_reason: _ } => {
                    
                },
                Event::ButtonSingleOrDoubleClickOrHold { click_type: ClickType::ButtonHold, conn_id: _, time_diff: _, was_queued: _ } => {
                    
                    let _ = trigger_channel.send(());
                },
                Event::GetInfoResponse { bd_addr_of_verified_buttons, .. } => {
                    let _ = tx.try_send(bd_addr_of_verified_buttons.clone());
                },
                _ => {}
            }
        });

        let flic_address = format!("{}:{}", self.ip, self.port);

        let client = FlicClient::new(&flic_address)
            .await?
            .register_event_handler(event)
            .await;
        let client1 = Arc::new(client);
        let client2 = client1.clone();


        let cmd_task = tokio::spawn({
            let client1 = Arc::clone(&client1);
            async move {
                
                client1.submit(FlicCommand::CreateScanWizard { scan_wizard_id: 1 }).await;
                
                sleep(Duration::from_secs(5)).await;
                
                client1.submit(FlicCommand::GetInfo).await;

                if let Some(verified_buttons) = rx.recv().await {
                    if !verified_buttons.is_empty() {
                        let mut conn_id = 1;
                        for bd_addr in verified_buttons.iter() {
                            client1.submit(FlicCommand::CreateConnectionChannel {
                                conn_id,
                                bd_addr: bd_addr.clone(),
                                latency_mode: LatencyMode::NormalLatency,
                                auto_disconnect_time: 511,
                            }).await;
                            conn_id += 1;
                            info!("[+] Flic Button {} is armed for DeadManSwitch!", bd_addr.clone());
                        }

                    } else {
                        error!("No verified buttons found");
                    }
                } else {
                    eprintln!("Failed to receive verified buttons");
                }
            }
        });

        let listen_task = tokio::spawn(async move {
            client2.listen().await;
            info!("Flic button client stopped.");
        });

        tokio::try_join!(cmd_task, listen_task)?;

        client1.stop().await;

        Ok(())
    }
}


struct DeadManSwitchAction;

impl DeadManSwitchAction {
    fn get_veracrypt_path(&self) -> String {
        if cfg!(windows) {
            "C:\\Program Files\\VeraCrypt\\VeraCrypt.exe".to_string()
        } else if cfg!(target_os = "macos") {
            "/Applications/VeraCrypt.app/Contents/MacOS/VeraCrypt".to_string()
        } else {
            "veracrypt".to_string()
        }
    }

    fn dismount_volumes(&self) {
        let veracrypt_path = self.get_veracrypt_path();

        let output = if cfg!(windows) {
            OsCommand::new(veracrypt_path)
                .args(&["/d", "/f", "/w", "/q", "/s"])
                .output()
        } else {
            OsCommand::new(veracrypt_path)
                .args(&["-d", "-f"])
                .output()
        };

        match output {
            Ok(_) => info!("[+] VeraCrypt volumes dismounted successfully."),
            Err(e) => error!("Error dismounting VeraCrypt volumes: {}", e),
        }
    }

    fn forced_hard_shutdown(&self) {
        let output = if cfg!(windows) {
            OsCommand::new("shutdown")
                .args(&["/p", "/f"])
                .output()
        } else if cfg!(target_os = "macos") {
            OsCommand::new("halt")
                .arg("-q")
                .output()
        } else {
            OsCommand::new("systemctl")
                .arg("poweroff")
                .arg("-f")
                .output()
        };

        match output {
            Ok(_) => info!("[+] System is performing a forced hard shutdown."),
            Err(e) => error!("Error performing a forced hard shutdown: {}", e),
        }
    }

    fn trigger_dms(&self) {
        warn!("[!] Dead Man Switch has been triggered.");
        warn!("[!] Sending Trigger message to the Local Network.");

        let message = CONFIG.broadcast_message.clone();
        let server_address: SocketAddr = format!("255.255.255.255:{}", CONFIG.broadcast_port).parse().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
        socket.set_broadcast(true).expect("Failed to set broadcast");
        socket.send_to(message.as_bytes(), server_address).expect("Failed to send message");
    }

    fn dead_man_switch(&self) {
        warn!("[!] Dismounting VeraCrypt volumes and locking the computer.");
        let action = self.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(3));
            action.dismount_volumes();
            action.forced_hard_shutdown();
            
        });
        let app = DeadManSwitchApp { remaining_seconds: 3, flash_emergency: true };
        app.run_alert();
    }
}

impl Clone for DeadManSwitchAction {
    fn clone(&self) -> Self {
        Self
    }
}



#[derive(Parser)]
#[clap(about = "Dead Man Switch")]
struct Args {
    #[clap(short, long, help = "Modes to run (comma-separated): net, bot, usb, flic, all", default_value = "all")]
    mode: String,

    #[clap(short, long, help = "Trigger the Dead Man Switch immediately")]
    trigger: bool,
}

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        LogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let args = Args::parse();
    let latest_message = Arc::new(Mutex::new(None));
    info!("[+] Activated Dead Man Switch");
    
    if args.trigger {
        DeadManSwitchAction.trigger_dms();
        //DeadManSwitchAction.dead_man_switch();
        std::process::exit(0);
    }

    let modes: Vec<&str> = args.mode.split(',').map(|s| s.trim()).collect();
    let run_all = modes.contains(&"all");

    let mut thread_handles = vec![];
    let mut task_handles = vec![];

    let (trigger_tx, trigger_rx) = std::sync::mpsc::channel();
    
    if run_all || modes.contains(&"net") {
        let network_listener = NetworkListener::new(CONFIG.broadcast_port, CONFIG.broadcast_message.clone(), Arc::clone(&latest_message));
        let broadcast_handle = thread::spawn(move || {
            network_listener.listen();
        });
        thread_handles.push(broadcast_handle);
    }

    if run_all || modes.contains(&"bot") {
        let bot = TelegramBot::new(CONFIG.telegram_bot_token.clone(), Arc::clone(&latest_message), "dms".to_string());
        let bot_handle = tokio::spawn(async move {
            bot.start().await;
        });
        task_handles.push(bot_handle);
    }

    if run_all || modes.contains(&"usb") {
        let usb_monitor = UsbMonitor::new(CONFIG.usb_vendor_id, CONFIG.usb_product_id);
        let usb_trigger_tx = trigger_tx.clone();
        let usb_handle = std::thread::spawn(move || {
            usb_monitor.monitor(usb_trigger_tx);
        });
        thread_handles.push(usb_handle);
    }


    if run_all || modes.contains(&"flic") {
        let flic_button = FlicButton::new(CONFIG.flic_ip.clone(), CONFIG.flic_port);
        let flic_trigger_tx = trigger_tx.clone();
        let flic_handle = tokio::spawn(async move {
            flic_button.handle(flic_trigger_tx).await.unwrap();
        });
        task_handles.push(flic_handle);
    }

    if thread_handles.is_empty() && task_handles.is_empty() {
        eprintln!("No valid modes provided. Available modes: net, bot, usb, all");
        std::process::exit(1);
    }

    #[cfg(target_os = "linux")]
    OsCommand::new("notify-send")
        .env("DISPLAY", ":0.0")
        .arg("Dead Man Switch 🏴‍☠️")
        .arg("The dead man's switch has been activated and armed. ⚔️")
        .output()
        .expect("Failed to execute notify-send command");

    #[cfg(target_os = "windows")]
    Notification::new()
        .summary("Dead Man Switch 🏴‍☠️")
        .body("The dead man's switch has been activated and armed. ⚔️")
        .timeout(0)
        .show();

    loop {
        if let Ok(_) = trigger_rx.try_recv() {
            let action = DeadManSwitchAction;
            action.trigger_dms();
            action.dead_man_switch();
            let app = DeadManSwitchApp { remaining_seconds: 3, flash_emergency: true };
            app.run_alert();
            std::process::exit(0);
        }

        thread::sleep(Duration::from_secs(1));
        let mut message = latest_message.lock().unwrap();
        if let Some(trigger_msg) = message.take() {
            match trigger_msg.source {
                MessageSource::Network => {
                    if trigger_msg.content.to_lowercase().contains(&CONFIG.broadcast_message) {
                        let action = DeadManSwitchAction;
                        action.trigger_dms();
                        action.dead_man_switch();
                        let app = DeadManSwitchApp { remaining_seconds: 3, flash_emergency: true };
                        app.run_alert();
                        std::process::exit(0);
                    }
                }
                MessageSource::Telegram => {
                    if trigger_msg.content.to_lowercase().contains(&CONFIG.telegram_command) {
                        let action = DeadManSwitchAction;
                        action.trigger_dms();
                        action.dead_man_switch();
                        let app = DeadManSwitchApp { remaining_seconds: 3, flash_emergency: true };
                        app.run_alert();
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}
