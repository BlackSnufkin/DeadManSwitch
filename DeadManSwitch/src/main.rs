// Uncomment this when you want to hide the window console
//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser};
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::window;
use iced::{executor, Application, Column, Command as IcedCommand, Container, Element, Length, Settings, Text};
use log::{error, info, warn, LevelFilter};
use rusb::{Context, Device, UsbContext, Error};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::process::Command as OsCommand;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::{Instant};
use teloxide::prelude::*;
use teloxide::types::{Message as BotMessage};
use teloxide::utils::command::BotCommands;


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
                IcedCommand::perform(wait_a_second(), Message::Tick),
                IcedCommand::perform(wait_half_second(), |_| Message::FlashEmergency),
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
                IcedCommand::perform(wait_a_second(), Message::Tick)
            }
            Message::FlashEmergency => {
                self.flash_emergency = !self.flash_emergency;
                IcedCommand::perform(wait_half_second(), |_| Message::FlashEmergency)
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

struct BlackBackgroundStyle;

impl iced::container::StyleSheet for BlackBackgroundStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: iced::Color::BLACK.into(),
            ..Default::default()
        }
    }
}

async fn wait_a_second() -> Instant {
    let duration = Duration::from_secs(1);
    std::thread::sleep(duration);
    Instant::now()
}

async fn wait_half_second() -> Instant {
    let duration = Duration::from_millis(275);
    std::thread::sleep(duration);
    Instant::now()
}

#[cfg(target_os = "windows")]
fn get_screen_size() -> (i32, i32) {
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
fn get_screen_size() -> (i32, i32) {
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
fn get_screen_size() -> (f64, f64) {
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
            return (rect.size.width, rect.size.height);
        }
    }
    (0.0, 0.0)
}


fn deadmanswitch_alert() {
    let size = get_screen_size();
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




#[derive(BotCommands, Clone)]
#[command(description = "These commands are supported:")]
enum Command {
    #[command(description = "Activate the Dead Man Switch.", rename = "dms")]
    Dms,
}


async fn start_bot(latest_message: Arc<Mutex<Option<String>>>) {
    let bot_token = "<TELEGRAM BOT API TOKEN>".to_string();
    let bot = Bot::new(bot_token);

    info!("[+] Telegram Bot Trigger is ON");

    let channel_latest_message = Arc::clone(&latest_message);
    let bot_clone = bot.clone(); 
    let channel_post_handler = Update::filter_channel_post().branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(move |cmd: Command, msg: BotMessage| {
                let latest_message = Arc::clone(&channel_latest_message);
                let bot = bot_clone.clone(); 
                async move {
                    
                    {
                        let mut message = latest_message.lock().unwrap();
                        *message = Some("dms".to_string());
                    } 
                    
                    if let Command::Dms = cmd {
                        let response_text = "Dead Man Switch activated! 🚨☠️";
                        bot.send_message(msg.chat.id, response_text).await?;
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


fn listen_for_broadcast(latest_message: Arc<Mutex<Option<String>>>) {
    let server_address: SocketAddr = "0.0.0.0:45370".parse().unwrap();
    let socket = UdpSocket::bind(server_address).expect("Failed to bind socket");
    info!("[+] Network Trigger is ON: {}", server_address);

    let mut buffer = [0; 4096];
    loop {
        let (size, _) = socket
            .recv_from(&mut buffer)
            .expect("Failed to receive data");

        let message = String::from_utf8_lossy(&buffer[..size]).to_string();
        if message.to_lowercase() == "dms" {
            let mut msg = latest_message.lock().unwrap();
            *msg = Some(message);
            break;
        }
        buffer = [0; 4096];
    }
}


fn get_veracrypt_path() -> String {
    if cfg!(windows) {
        "C:\\Program Files\\VeraCrypt\\VeraCrypt.exe".to_string()
    } else if cfg!(target_os = "macos") {
        "/Applications/VeraCrypt.app/Contents/MacOS/VeraCrypt".to_string()
    } else {
        "veracrypt".to_string()
    }
}


fn dismount_veracrypt_volumes() {
    let veracrypt_path = get_veracrypt_path();

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

fn forced_hard_shutdown() {
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


fn trigger_dms() {
    warn!("[!] Dead Man Switch has been triggered.");
    warn!("[!] Sending Trigger message to the Local Network.");

    let message = "dms";
    let server_address: SocketAddr = "255.255.255.255:45370".parse().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    socket.set_broadcast(true).expect("Failed to set broadcast");
    socket
        .send_to(message.as_bytes(), server_address)
        .expect("Failed to send message");
}

fn dead_man_switch() {
    warn!("[!] Dead Man Switch has been triggered.");
    warn!("[!] Dismounting VeraCrypt volumes and locking the computer.");
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        dismount_veracrypt_volumes();
        forced_hard_shutdown();
    });
    deadmanswitch_alert();
}

fn monitor_usb_devices() {
    let context = Context::new().expect("Failed to create USB context");
    let mut known_devices: HashSet<DeviceInfo> = HashSet::new();
    
    info!("[+] USB Device Trigger is ON");
    loop {
        let devices = context.devices().expect("Failed to get USB devices");
        for device in devices.iter() {
            let device_info = DeviceInfo::from_device(&device).expect("Failed to get device info");
            if known_devices.insert(device_info.clone()) {
                
                if usb_trigger(&device).expect("Failed to check dead man switch") {
                    trigger_dms();
                    dead_man_switch();
                    std::process::exit(0);
                }
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct DeviceInfo {
    vendor_id: u16,
    product_id: u16,
    bus_number: u8,
    device_address: u8,
}

impl DeviceInfo {
    fn from_device<T: UsbContext>(device: &Device<T>) -> Result<Self, Error> {
        let device_desc = device.device_descriptor()?;
        Ok(Self {
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),
            bus_number: device.bus_number(),
            device_address: device.address(),
        })
    }
}

fn usb_trigger<T: UsbContext>(device: &Device<T>) -> Result<bool, Error> {
    let device_desc = device.device_descriptor()?;
    Ok(device_desc.vendor_id() == 0x090c && device_desc.product_id() == 0x1000)
}



#[derive(clap::Parser)]
#[clap(about = "Dead Man Switch")]
struct Args {
    #[clap(short, long, help = "Modes to run (comma-separated): net, bot, usb, all",default_value = "all")]
    mode: String,
    
    #[clap(short, long, help = "Trigger the Dead Man Switch immediately")]
    trigger: bool,
}


#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();


    let args = Args::parse();
    let latest_message = Arc::new(Mutex::new(None));
    info!("[+] Activated Dead Man Switch");

    let modes: Vec<&str> = args.mode.split(',').map(|s| s.trim()).collect();

    let mut thread_handles = vec![];
    let mut task_handles = vec![];

    let run_all = modes.contains(&"all");

    if run_all || modes.contains(&"net") {
        let broadcast_latest_message = Arc::clone(&latest_message);
        let broadcast_handle = thread::spawn(move || {
            listen_for_broadcast(broadcast_latest_message);
        });
        thread_handles.push(broadcast_handle);
    }

    if run_all || modes.contains(&"bot") {
        let bot_latest_message = Arc::clone(&latest_message);
        let bot_handle = tokio::spawn(async move {
            start_bot(bot_latest_message).await;
        });
        task_handles.push(bot_handle);
    }

    if run_all || modes.contains(&"usb") {
        let usb_handle = thread::spawn(move || {
            monitor_usb_devices();
        });
        thread_handles.push(usb_handle);
    }

    if thread_handles.is_empty() && task_handles.is_empty() {
        eprintln!("No valid modes provided. Available modes: net, bot, usb, all");
        std::process::exit(1);
    }

    use notify_rust::{Notification};
    Notification::new()
        .summary("Dead Man Switch 🏴‍☠️")
        .body("The dead man's switch has been activated and armed. ⚔️")
        .timeout(0) // this however is
        .show();

    if args.trigger {
        trigger_dms();
        dead_man_switch();
        std::process::exit(0);
    }

    loop {
        thread::sleep(Duration::from_secs(1));
        let mut message = latest_message.lock().unwrap();
        if let Some(msg) = message.take() {
            if msg.to_lowercase().contains("dms") {
                trigger_dms();
                dead_man_switch();
                std::process::exit(0);
            }
        }
    }
}

