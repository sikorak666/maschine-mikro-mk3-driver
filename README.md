# Maschine Mikro MK3 MIDI Linux Driver (work in progress)
Native Instruments Maschine Mikro MK3 userspace MIDI driver for Linux.

This project is a fork of the original [`maschine-mikro-mk3-driver`](https://github.com/r00tman/maschine-mikro-mk3-driver.git) by **r00tman**.  
**Many thanks to r00tman** for their foundational work — without it, this continuation would not have been possible.

This Rust-based project provides a **custom driver** for the **Native Instruments Maschine Mikro MK3** controller.  
Its primary goal is to enable **advanced configuration** far beyond the default capabilities.

---

## Key Features

- **File-Based Configuration**  
  All settings are loaded from a `maschine_config.json` file — easy to edit without recompiling.

- **Pad Velocity Sensitivity**  
  Adjust the sensitivity of the pads using the `velocity_sensitivity` parameter in the config.

- **Pad MIDI Mapping (Note Maps)**  
  Customize the MIDI note for each of the 16 pads individually.

- **Pad Color & Backlight Settings**
  - `pad_colors`: Set the color of each pad using a wide range of hues.
  - `backlight` settings:
    - `note_off`: `"off"`, `"dim"`, `"normal"`, or `"bright"`
    - `note_on`: `"off"`, `"dim"`, `"normal"`, or `"bright"`

This project transforms the Mikro MK3 into a **flexible, personalizable MIDI controller**.

---

##  Installation & Setup

### 1. Install Dependencies

```bash
sudo apt install cargo pkg-config build-essential libusb-1.0-0-dev libudev-dev
```

### 2. Clone and Build
```bash
git clone https://github.com/r00tman/maschine-mikro-mk3-driver.git
cd maschine-mikro-mk3-driver
sudo cp 98-maschine.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
cargo build --release
```

### 3. Set Up User Permissions
Ensure your user is part of the `input` group:
```bash
sudo usermod -a -G input $USER
```
You may need to log out and log back in for changes to take effect.
If your user is in the input group, the controller should initialize and create an ALSA MIDI port named:**Maschine Mikro Mk3 MIDI**

### 4. (Optional) Create a Convenient Alias
To run the driver from anywhere using a simple command:
```bash
echo "alias maschine='/home/studio/maschine-mikro-mk3-driver/target/release/maschine-mikro-mk3-driver'" >> ~/.bashrc
source ~/.bashrc
```
Now you can just type **maschine** in your terminal to launch the driver.


## Next Steps / Planned Features

    ✅ MIDI Assignment for Buttons & Encoders
    Assign custom MIDI messages to physical controls via the config file.

    ✅ Button Backlight Control
    Fully customizable button backlighting and behavior.

    🚧 GUI Settings Generator
    Build a simple GUI to manage and preview maschine_config.json settings.

## Credits
    Original work by r00tman
    Forked and extended for advanced configuration and usability

Contributions are welcome!

## Goal

The current goal is to reimplement the official MIDI Mode: mappable pads, buttons, slider, encoder, changeable LED color schemes.
Advanced uses, like modal functions as in Maschine software (e.g., Scenes, Patterns, Shift+Pad actions) are not yet planned.
