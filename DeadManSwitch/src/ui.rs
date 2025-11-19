use eframe::egui;
use std::time::{Duration, Instant};

pub struct AlertWindow {
    remaining: i32,
    flash: bool,
    last_tick: Instant,
    last_flash: Instant,
}

impl Default for AlertWindow {
    fn default() -> Self {
        Self {
            remaining: 3,
            flash: true,
            last_tick: Instant::now(),
            last_flash: Instant::now(),
        }
    }
}

impl eframe::App for AlertWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Timer logic
        if self.last_tick.elapsed() >= Duration::from_secs(1) {
            if self.remaining > 0 {
                self.remaining -= 1;
            }
            self.last_tick = Instant::now();
        }

        // Flash logic
        if self.last_flash.elapsed() >= Duration::from_millis(500) {
            self.flash = !self.flash;
            self.last_flash = Instant::now();
        }

        // Request continuous repaint for animation
        ctx.request_repaint();

        // Fullscreen black background
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::BLACK)
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);

                    // Flashing title
                    let title_color = if self.flash {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::RED
                    };

                    ui.label(
                        egui::RichText::new("!!! DEAD MAN SWITCH !!!")
                            .size(100.0)
                            .color(title_color)
                            .strong()
                    );

                    ui.add_space(80.0);

                    // Message
                    ui.label(
                        egui::RichText::new(
                            "CRITICAL SECURITY ALERT\n\n\
                            System shutdown in progress\n\
                            All encrypted volumes will be dismounted"
                        )
                        .size(50.0)
                        .color(egui::Color32::WHITE)
                    );

                    ui.add_space(100.0);

                    // Countdown
                    ui.label(
                        egui::RichText::new(format!("Shutdown in: {} sec", self.remaining))
                            .size(60.0)
                            .color(egui::Color32::RED)
                            .strong()
                    );
                });
            });
    }
}

fn get_screen_size() -> (f32, f32) {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as f32;
            let height = GetSystemMetrics(SM_CYSCREEN) as f32;
            return (width, height);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use x11::xlib;
        unsafe {
            let display = xlib::XOpenDisplay(std::ptr::null());
            if !display.is_null() {
                let screen = xlib::XDefaultScreen(display);
                let width = xlib::XDisplayWidth(display, screen) as f32;
                let height = xlib::XDisplayHeight(display, screen) as f32;
                xlib::XCloseDisplay(display);
                return (width, height);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSScreen;
        use cocoa::base::nil;
        unsafe {
            let screen = NSScreen::mainScreen(nil);
            if screen != nil {
                let rect = NSScreen::frame(screen);
                return (rect.size.width as f32, rect.size.height as f32);
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        (1920.0, 1080.0)
    }
}

pub fn show_alert() {
    let (width, height) = get_screen_size();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_fullscreen(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "DEAD MAN SWITCH",
        options,
        Box::new(|_cc| Box::new(AlertWindow::default())),
    );
}