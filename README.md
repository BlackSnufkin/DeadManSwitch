# Dead Man Switch

![dms_logo](https://github.com/BlackSnufkin/DeadManSwitch/assets/61916899/097bc494-f245-4b0a-8a71-9971708fc98e)

## Overview

Dead Man Switch is a cross-platform security application written in Rust that automatically protects sensitive data in emergency situations. The application monitors multiple trigger mechanisms and executes immediate system lockdown procedures when activated, including encrypted volume dismounting and controlled system shutdown.

### Core Features

- **Heartbeat Timer**: Automatic trigger on missed check-in intervals
- **Telegram Integration**: Remote control via secure messaging
- **Network Triggers**: LAN broadcast-based activation
- **USB Detection**: Hardware-based triggering
- **Flic Button Support**: Physical activation mechanism
- **VeraCrypt Integration**: Automatic encrypted volume dismounting
- **Cross-Platform**: Windows, Linux, and macOS compatibility

## Requirements

- Rust 1.82.0 or later ([Installation Guide](https://rustup.rs/))
- VeraCrypt ([Download](https://www.veracrypt.fr/))
- Telegram Bot Token (optional, for remote features)


## Configuration

Edit `src/config.rs` to customize settings:
```rust
pub fn default() -> Result<Self> {
    Self::new(
        "YOUR_TELEGRAM_BOT_TOKEN".to_string(),
        3600,                                     // Heartbeat timeout (seconds)
        45370,                                    // Network broadcast port
        "trigger_dms".to_string(),               // Network trigger message
        "execute".to_string(),                   // Manual trigger command
        0x090c,                                  // USB vendor ID
        0x1000,                                  // USB product ID
        "auto".to_string(),                      // Flic IP (auto-detected)
        5551,                                    // Flic port
    )
}
```

### Obtaining Telegram Bot Token

1. Contact `@BotFather` on Telegram
2. Execute `/newbot` command
3. Follow setup instructions
4. Copy provided API token
5. Insert token into configuration file

## Usage

Basic form:

    ./DeadManSwitch --mode <modes> [--trigger]

- `--mode`: comma-separated list of modes. If omitted, `all` is used.
- `--trigger`: execute actions immediately and show the alert UI, without waiting for any external trigger.

### Modes

Available modes:

- `timer` – Telegram heartbeat timer
- `bot`   – Telegram manual trigger
- `net`   – UDP broadcast listener
- `usb`   – USB VID/PID trigger
- `flic`  – Flic button trigger
- `all`   – All of the above

Examples:

    # All triggers (default)
    ./DeadManSwitch

    # Only Telegram heartbeat + USB
    ./DeadManSwitch --mode timer,usb

    # Only network broadcast
    ./DeadManSwitch --mode net

    # Manual trigger only
    ./DeadManSwitch --trigger


## Trigger Mechanisms

### 1. Heartbeat Timer

Automatically triggers after specified timeout period without check-in.

**Configuration:**
```rust
telegram_heartbeat_timeout: 3600  // seconds
```

**Execution:**
```bash
./DeadManSwitch --mode timer
```

**Telegram Commands:**
- `/alive` - Reset countdown timer
- `/status` - Query time remaining

**Behavior:**
- Monitors for periodic heartbeat signals
- Sends Telegram alert on timeout
- Executes lockdown procedures automatically



### 2. Telegram Bot

Remote manual control interface.

**Setup:**
1. Create bot via @BotFather
2. Add bot to channel with "Post Messages" permission
3. Update configuration with token

**Execution:**
```bash
./DeadManSwitch --mode bot
```

**Command:**
- `/dms execute` - Manual trigger activation



### 3. Network Broadcast

LAN-based triggering mechanism.

**Configuration:**
```rust
broadcast_port: 45370
broadcast_message: "trigger_dms"
```

**Execution:**
```bash
# Protected machine
./DeadManSwitch --mode net

# Trigger from network
echo "trigger_dms" | nc -u -b 255.255.255.255 45370
```



### 4. USB Device Detection

Hardware-based activation on device connection.

**Identifying Device IDs:**
```bash
git clone https://github.com/BlackSnufkin/Rusty-Playground.git
cd Rusty-Playground/USB_mon
cargo run
# Connect device and note vendor:product IDs
```

**Configuration:**
```rust
usb_vendor_id: 0x090c
usb_product_id: 0x1000
```

**Execution:**
```bash
./DeadManSwitch --mode usb
```

---

### 5. Flic Button

Physical button activation mechanism.

**Server Installation:**
- Linux: [Flic SDK for Linux](https://github.com/50ButtonsEach/fliclib-linux-hci)
- Windows: [Flic Windows SDK](https://github.com/50ButtonsEach/fliclib-windows)

**Button Pairing:**
```bash
./simpleclient scan
./simpleclient connect <button_address>
```

**Configuration:**
```rust
flic_ip: "192.168.1.242".to_string()  // or "auto" for detection
```

**Execution:**
```bash
./DeadManSwitch --mode flic
```

**Activation:** Press and hold button


## Recommended Use with VeraCrypt

- Encrypt your volumes/partitions with VeraCrypt
- Configure `ActionExecutor` to dismount and shutdown on trigger
- Test everything on non-critical systems before using it on real data


## Disclaimer

- Provided without any warranty
- You are responsible for any data loss or damage
- Always keep backups and test configuration before using this on important systems
