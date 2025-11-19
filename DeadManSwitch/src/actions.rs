use std::process::Command;
use std::thread;
use std::time::Duration;
use crate::config::Config;
use crate::triggers::network::NetworkListener;

pub struct ActionExecutor {
    config: Config,
}

impl ActionExecutor {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn execute(&self) {
        log::warn!("[!] Dead Man Switch ACTIVATED");
        
        // Broadcast to network
        if let Err(e) = NetworkListener::send_trigger_broadcast(&self.config) {
            log::error!("Broadcast failed: {}", e);
        }

        // Spawn async dismount/shutdown
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(3));
            Self::dismount_veracrypt();
            Self::force_shutdown();
        });
    }

    fn veracrypt_path() -> &'static str {
        if cfg!(windows) {
            "C:\\Program Files\\VeraCrypt\\VeraCrypt.exe"
        } else if cfg!(target_os = "macos") {
            "/Applications/VeraCrypt.app/Contents/MacOS/VeraCrypt"
        } else {
            "veracrypt"
        }
    }

    fn dismount_veracrypt() {
        let args = if cfg!(windows) {
            vec!["/d", "/f", "/w", "/q", "/s"]
        } else {
            vec!["-d", "-f"]
        };

        match Command::new(Self::veracrypt_path()).args(&args).output() {
            Ok(_) => log::info!("[+] VeraCrypt dismounted"),
            Err(e) => log::error!("VeraCrypt error: {}", e),
        }
    }

    fn force_shutdown() {
        let result = if cfg!(windows) {
            Command::new("shutdown").args(&["/p", "/f"]).output()
        } else if cfg!(target_os = "macos") {
            Command::new("halt").arg("-q").output()
        } else {
            Command::new("systemctl").args(&["poweroff", "-f"]).output()
        };

        match result {
            Ok(_) => log::info!("[+] System shutdown initiated"),
            Err(e) => log::error!("Shutdown error: {}", e),
        }
    }

    pub fn send_notification(modes: &[&str]) {
        let msg = format!("DMS armed ({})", modes.join(", "));
        
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("notify-send")
                .env("DISPLAY", ":0.0")
                .args(&["Dead Man Switch 🏴‍☠️", &msg])
                .output();
        }
        
        #[cfg(target_os = "windows")]
        {
            use notify_rust::Notification;
            let _ = Notification::new()
                .summary("Dead Man Switch 🏴‍☠️")
                .body(&msg)
                .show();
        }
        
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("osascript")
                .arg("-e")
                .arg(&format!("display notification \"{}\" with title \"DMS\"", msg))
                .output();
        }
    }
}