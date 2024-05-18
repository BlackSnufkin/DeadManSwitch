# Dead Man Switch

A Rust application that activates a Dead Man Switch to secure your computer in case of an emergency. The application supports multiple trigger mechanisms, including a Telegram bot, network broadcasts, and USB device detection. When triggered, the Dead Man Switch dismounts VeraCrypt volumes and performs a forced hard shutdown of the system.

## Features

- Trigger the Dead Man Switch using a Telegram bot, network broadcasts, or by connecting a specific USB device
- Dismount VeraCrypt volumes to secure confidential data
- Perform a forced hard shutdown of the system
- Display an emergency alert window with a countdown timer
- Customizable trigger commands and USB device detection

## Prerequisites

- Rust programming language (1.55.0 or later)
- VeraCrypt installed on the system
- Telegram Bot API token (for the Telegram bot trigger)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/yourusername/dead-man-switch.git
   cd dead-man-switch
   ```

2. Update the Telegram Bot API token in the code:

   Replace `<TELEGRAM BOT API TOKEN>` with your actual bot token in the `start_bot` function.

3. Build the application:

   ```bash
   cargo build --release
   ```

## Usage

Run the application with the desired trigger mode:

```bash
./target/release/dead-man-switch --mode <mode>
```

Available modes:
- `net`: Listen for network broadcasts to trigger the Dead Man Switch
- `bot`: Use a Telegram bot to trigger the Dead Man Switch
- `usb`: Monitor USB devices and trigger the Dead Man Switch when a specific device is connected
- `all`: Enable all trigger modes (default)

To manually trigger the Dead Man Switch, use the `--trigger` flag:

```bash
./target/release/dead-man-switch --trigger
```

## Customization

### Telegram Bot Command
- Modify the `Command` enum in the code to change the Telegram bot command that triggers the Dead Man Switch.
- Update the corresponding command handler in the `start_bot` function to match the new command.

### Network Broadcast Message
- Change the expected broadcast message in the `listen_for_broadcast` and in `trigger_dms` functions to customize the message that triggers the Dead Man Switch over the network.


### USB Device Detection
- Use the code from the [USB_mon](https://github.com/BlackSnufkin/Rusty-Playground/tree/main/USB_mon) project to find the vendor and product IDs of the desired USB device.
- Update the `usb_trigger` function in the Dead Man Switch code with the specific vendor and product IDs to customize the USB device detection.

## Dependencies

- `iced`: GUI library for Rust
- `log` and `simplelog`: Logging functionality
- `teloxide`: Telegram bot library
- `clap`: Command-line argument parsing
- `rusb`: USB device detection and communication
- `winapi`, `x11`, and `cocoa`: System-specific libraries for screen size detection

## License

This project is licensed under the [MIT License](LICENSE).

## Disclaimer

This application is intended for educational and testing purposes only. Use it responsibly and ensure that you have the necessary permissions and legal rights to perform actions like dismounting volumes and shutting down the system. The authors of this project are not responsible for any misuse or damage caused by this application.
