
![dms_logo](https://github.com/BlackSnufkin/DeadManSwitch/assets/61916899/097bc494-f245-4b0a-8a71-9971708fc98e)



# Dead Man Switch

A Rust application that activates a Dead Man Switch to secure your computer in case of an emergency. The application supports multiple trigger mechanisms, including a Telegram bot, network broadcasts, and USB device detection. When triggered, the Dead Man Switch dismounts VeraCrypt volumes and performs a forced hard shutdown of the system.

## Features

- Trigger the Dead Man Switch using a Telegram bot, network broadcasts, or by connecting a specific USB device
- Dismount VeraCrypt volumes to secure confidential data
- Perform a forced hard shutdown of the system
- Display an emergency alert window with a countdown timer
- Customizable trigger commands and USB device detection
- Cross-platform compatibility worked on Debian and Windows systems, expected to work on macOS (didnt tested yet)

## Recommendations

For optimal data protection, it is highly recommended to encrypt your system partition using VeraCrypt. This ensures that all your sensitive data remains secure even if the computer is compromised or stolen. By combining the Dead Man Switch application with a VeraCrypt-encrypted system partition, you can achieve a robust security setup that safeguards your data in case of an emergency.

To encrypt your system partition with VeraCrypt, please follow the official VeraCrypt documentation and guides available on their website.

## Prerequisites

- Rust programming language (1.55.0 or later)
- VeraCrypt installed on the system
- Telegram Bot API token (for the Telegram bot trigger)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/BlackSnufkin/DeadManSwitch.git
   cd DeadManSwitch
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
./target/release/DeadManSwitch --mode <mode>
```

Available modes:
- `net`: Listen for network broadcasts to trigger the Dead Man Switch
- `bot`: Use a Telegram bot to trigger the Dead Man Switch
- `usb`: Monitor USB devices and trigger the Dead Man Switch when a specific device is connected
- `all`: Enable all trigger modes (default)

To manually trigger the Dead Man Switch, use the `--trigger` flag:

```bash
./target/release/DeadManSwitch --trigger
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

- This application is provided "as is" without any warranty, express or implied. The authors of this project shall not be held liable for any damage, data loss, or other consequences arising from the use or misuse of this application.
- By using this application, you acknowledge that you are using it at your own risk. 
- The authors of this project cannot be held responsible for any corruption of VeraCrypt containers, data loss, system instability, or any other adverse effects that may occur due to the use of this application.
- Use this application wisely and exercise caution. Always maintain backups of your important data and ensure that you have a thorough understanding of the application's functionality before using it on critical systems.
