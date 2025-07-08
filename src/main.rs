mod controls;
mod font;
mod lights;
mod screen;
use crate::controls::{Buttons, PadEventType};
use crate::font::Font;
use crate::lights::{Brightness, Lights, PadColors};
use crate::screen::Screen;
use hidapi::{HidDevice, HidResult};
use midly::{live::LiveEvent, MidiMessage};
use std::{thread, time};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PadBrightness {
    #[serde(default = "default_brightness_off")]
    note_off: String,
    #[serde(default = "default_brightness_on")]
    note_on: String,
}

impl Default for PadBrightness {
    fn default() -> Self {
        PadBrightness {
            note_off: default_brightness_off(),
            note_on: default_brightness_on(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_velocity_sensitivity")]
    velocity_sensitivity: f32,
    #[serde(default = "default_note_maps")]
    note_maps: [u8; 16],
    #[serde(default)]
    pad_brightness: PadBrightness,
    #[serde(default = "default_pad_colors")]
    pad_colors: [String; 16],
}

impl Default for Config {
    fn default() -> Self {
        Config {
            velocity_sensitivity: default_velocity_sensitivity(),
            note_maps: default_note_maps(),
            pad_brightness: PadBrightness::default(),
            pad_colors: default_pad_colors(),
        }
    }
}

fn default_note_maps() -> [u8; 16] {
    [49, 27, 31, 57, 48, 47, 43, 59, 36, 38, 46, 51, 36, 38, 42, 44]
}

fn default_velocity_sensitivity() -> f32 {
    1.0
}

fn default_brightness_off() -> String {
    "off".to_string()
}

fn default_brightness_on() -> String {
    "normal".to_string()
}

fn default_pad_colors() -> [String; 16] {
    [
        "white".to_string(), "white".to_string(), "white".to_string(), "white".to_string(),
        "white".to_string(), "white".to_string(), "white".to_string(), "white".to_string(),
        "white".to_string(), "white".to_string(), "white".to_string(), "white".to_string(),
        "white".to_string(), "white".to_string(), "white".to_string(), "white".to_string(),
    ]
}

fn string_to_brightness(s: &str) -> Brightness {
    match s.to_lowercase().as_str() {
        "off" => Brightness::Off,
        "dim" => Brightness::Dim,
        "normal" => Brightness::Normal,
        "bright" => Brightness::Bright,
        _ => Brightness::Normal,
    }
}

fn string_to_pad_color(s: &str) -> PadColors {
    match s.to_lowercase().as_str() {
        "off" => PadColors::Off,
        "red" => PadColors::Red,
        "orange" => PadColors::Orange,
        "lightorange" => PadColors::LightOrange,
        "warmyellow" => PadColors::WarmYellow,
        "yellow" => PadColors::Yellow,
        "lime" => PadColors::Lime,
        "green" => PadColors::Green,
        "mint" => PadColors::Mint,
        "cyan" => PadColors::Cyan,
        "turquoise" => PadColors::Turquoise,
        "blue" => PadColors::Blue,
        "plum" => PadColors::Plum,
        "violet" => PadColors::Violet,
        "purple" => PadColors::Purple,
        "magenta" => PadColors::Magenta,
        "fuchsia" => PadColors::Fuchsia,
        "white" => PadColors::White,
        _ => PadColors::White,
    }
}

fn get_config_path() -> PathBuf {
    // Opcja 1: Sprawdź zmienną środowiskową
    if let Ok(custom_path) = env::var("MASCHINE_CONFIG_PATH") {
        return PathBuf::from(custom_path);
    }

    // Opcja 2: Katalog z plikiem wykonywalnym
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let config_in_exe_dir = exe_dir.join("maschine_config.json");
            // Jeśli plik już istnieje tam, używaj go
            if config_in_exe_dir.exists() {
                return config_in_exe_dir;
            }
            // Jeśli mamy uprawnienia do zapisu, używaj tego katalogu
            if exe_dir.metadata().map(|m| !m.permissions().readonly()).unwrap_or(false) {
                return config_in_exe_dir;
            }
        }
    }

    // Opcja 3: Katalog ~/.config/maschine-mikro-mk3/
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir = home_dir.join(".config").join("maschine-mikro-mk3");
        // Utwórz katalog jeśli nie istnieje
        let _ = fs::create_dir_all(&config_dir);
        return config_dir.join("maschine_config.json");
    }

    // Opcja 4: Bieżący katalog (fallback)
    PathBuf::from("maschine_config.json")
}

fn load_config() -> Config {
    let config_path = get_config_path();
    println!("Szukam pliku konfiguracji: {}", config_path.display());

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(config_str) => {
                match serde_json::from_str(&config_str) {
                    Ok(config) => {
                        println!("Załadowano konfigurację z: {}", config_path.display());
                        config
                    }
                    Err(e) => {
                        eprintln!("Błąd parsowania pliku konfiguracji: {}", e);
                        eprintln!("Używam domyślnej konfiguracji");
                        Config::default()
                    }
                }
            }
            Err(e) => {
                eprintln!("Błąd odczytu pliku konfiguracji: {}", e);
                eprintln!("Używam domyślnej konfiguracji");
                Config::default()
            }
        }
    } else {
        println!("Plik konfiguracji nie istnieje.");
        println!("Tworzę domyślny plik konfiguracji...");

        let default_config = Config::default();
        match serde_json::to_string_pretty(&default_config) {
            Ok(json) => {
                // Upewnij się, że katalog istnieje
                if let Some(parent) = config_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                if let Err(e) = fs::write(&config_path, json) {
                    eprintln!("Nie udało się utworzyć pliku konfiguracji: {}", e);
                } else {
                    println!("Utworzono domyślny plik konfiguracji: {}", config_path.display());
                }
            }
            Err(e) => {
                eprintln!("Błąd serializacji konfiguracji: {}", e);
            }
        }

        default_config
    }
}

fn self_test(device: &HidDevice, screen: &mut Screen, lights: &mut Lights) -> HidResult<()> {
    Font::write_digit(screen, 0, 0, 1, 4);
    screen.write(&device)?;
    thread::sleep(time::Duration::from_millis(100));
    Font::write_digit(screen, 0, 32, 3, 4);
    screen.write(&device)?;
    thread::sleep(time::Duration::from_millis(100));
    Font::write_digit(screen, 0, 64, 3, 4);
    screen.write(&device)?;
    thread::sleep(time::Duration::from_millis(100));
    Font::write_digit(screen, 0, 96, 7, 4);
    screen.write(&device)?;

    for i in 0..39 {
        lights.set_button(num::FromPrimitive::from_u32(i).unwrap(), Brightness::Bright);
        lights.write(&device)?;
        lights.set_button(num::FromPrimitive::from_u32(i).unwrap(), Brightness::Normal);
        lights.write(&device)?;
        lights.set_button(num::FromPrimitive::from_u32(i).unwrap(), Brightness::Dim);
        lights.write(&device)?;
        // thread::sleep(time::Duration::from_millis(100));
    }
    for i in 0..16 {
        // let color: PadColors = PadColors::Blue;
        let color: PadColors = num::FromPrimitive::from_usize(i + 2).unwrap();
        lights.set_pad(i, color, Brightness::Bright);
        lights.write(&device)?;
        let color: PadColors = num::FromPrimitive::from_usize(i + 1).unwrap();
        lights.set_pad(i, color, Brightness::Normal);
        lights.write(&device)?;
        let color: PadColors = num::FromPrimitive::from_usize(i + 1).unwrap();
        lights.set_pad(i, color, Brightness::Dim);
        lights.write(&device)?;
        // thread::sleep(time::Duration::from_millis(1000));
    }
    for i in 0..25 {
        lights.set_slider(i, Brightness::Bright);
        lights.write(&device)?;
        lights.set_slider(i, Brightness::Normal);
        lights.write(&device)?;
        lights.set_slider(i, Brightness::Dim);
        lights.write(&device)?;
        // thread::sleep(time::Duration::from_millis(1000));
    }
    lights.reset();
    lights.write(&device)?;

    screen.reset();
    screen.write(&device)?;

    Ok(())
}

fn main() -> HidResult<()> {
    // Ładowanie konfiguracji z pliku
    let config = load_config();
    let notemaps = config.note_maps;
    let velocity_sensitivity = config.velocity_sensitivity;
    let brightness_off = string_to_brightness(&config.pad_brightness.note_off);
    let brightness_on = string_to_brightness(&config.pad_brightness.note_on);
    let pad_colors: Vec<PadColors> = config.pad_colors.iter()
    .map(|c| string_to_pad_color(c))
    .collect();

    println!("Używam mapowania MIDI: {:?}", notemaps);
    println!("Czułość velocity: {}", velocity_sensitivity);
    println!("Jasność padów: {} -> {}", config.pad_brightness.note_off, config.pad_brightness.note_on);

    let output = MidiOutput::new("Maschine Mikro MK3").expect("Couldn't open MIDI output");
    let mut port = output.create_virtual("Maschine Mikro MK3 MIDI Out").expect("Couldn't create virtual port");

    let api = hidapi::HidApi::new()?;
    #[allow(non_snake_case)]
    let (VID, PID) = (0x17cc, 0x1700);
    let device = api.open(VID, PID)?;

    device.set_blocking_mode(false)?;

    let mut screen = Screen::new();
    let mut lights = Lights::new();

    self_test(&device, &mut screen, &mut lights)?;

    // Inicjalizacja padów z konfiguracji
    for i in 0..16 {
        lights.set_pad(i, pad_colors[i], brightness_off);
    }
    lights.write(&device)?;

    let mut buf = [0u8; 64];
    loop {
        let size = device.read_timeout(&mut buf, 10)?;
        if size < 1 {
            continue;
        }

        let mut changed_lights = false;
        if buf[0] == 0x01 {
            // button mode
            for i in 0..6 {
                // bytes
                for j in 0..8 {
                    // bits
                    let idx = i * 8 + j;
                    let button: Option<Buttons> = num::FromPrimitive::from_usize(idx);
                    let button = match button {
                        Some(val) => val,
                        None => continue,
                    };
                    let status = buf[i + 1] & (1 << j);
                    let status = status > 0;
                    if status {
                        println!("{:?}", button);
                    }
                    if lights.button_has_light(button) {
                        let light_status = lights.get_button(button) != Brightness::Off;
                        if status != light_status {
                            lights.set_button(
                                button,
                                if status {
                                    Brightness::Normal
                                } else {
                                    Brightness::Dim
                                },
                            );
                            changed_lights = true;
                        }
                    }
                }
            }
            let encoder_val = buf[7];
            println!("Encoder: {}", encoder_val);
            let slider_val = buf[10];
            if slider_val != 0 {
                println!("Slider: {}", slider_val);
                let cnt = (slider_val as i32 - 1 + 5) * 25 / 200 - 1;
                for i in 0..25 {
                    let b = match cnt - i {
                        0 => Brightness::Normal,
                        1..=25 => Brightness::Dim,
                        _ => Brightness::Off,
                    };
                    lights.set_slider(i as usize, b);
                }
                changed_lights = true;
            }
        } else if buf[0] == 0x02 {
            // pad mode
            for i in (1..buf.len()).step_by(3) {
                let idx = buf[i];
                let evt = buf[i + 1] & 0xf0;
                let val = ((buf[i + 1] as u16 & 0x0f) << 8) + buf[i + 2] as u16;
                if i > 1 && idx == 0 && evt as u8 == 0 && val == 0 {
                    break;
                }
                let pad_evt: PadEventType = num::FromPrimitive::from_u8(evt).unwrap();
                // if evt != PadEventType::Aftertouch {
                println!("Pad {}: {:?} @ {}", idx, pad_evt, val);
                // }
                let (_, prev_b) = lights.get_pad(idx as usize);
                let b = match pad_evt {
                    PadEventType::NoteOn | PadEventType::PressOn => brightness_on,
                    PadEventType::NoteOff | PadEventType::PressOff => brightness_off,
                    PadEventType::Aftertouch => {
                        if val > 0 {
                            brightness_on
                        } else {
                            brightness_off
                        }
                    }
                    #[allow(unreachable_patterns)]
                    _ => prev_b,
                };
                if prev_b != b {
                    lights.set_pad(idx as usize, pad_colors[idx as usize], b);
                    changed_lights = true;
                }
                // let padids = [13, 14, 15, 16, 9, 10, 11, 12, 5, 6, 7, 8, 1, 2, 3, 4];
                // let note = padids[idx as usize]-1+36;

                let note = notemaps[idx as usize];

                // Skalowanie velocity z konfiguracją czułości
                let raw_velocity = (val as f32 * velocity_sensitivity / 32.0).min(127.0) as u8;
                let velocity = if raw_velocity > 0 && val > 0 {
                    raw_velocity.max(1)
                } else {
                    0
                };

                let event = match pad_evt {
                    PadEventType::NoteOn | PadEventType::PressOn => Some(MidiMessage::NoteOn {
                        key: note.into(),
                                                                         vel: velocity.into(),
                    }),
                    PadEventType::NoteOff | PadEventType::PressOff => Some(MidiMessage::NoteOff {
                        key: note.into(),
                                                                           vel: velocity.into(),
                    }),
                    _ => {None}
                };

                if let Some(evt) = event {
                    let l_ev = LiveEvent::Midi {
                        channel: 0.into(),
                        message: evt,
                    };
                    let mut buf = Vec::new();
                    l_ev.write(&mut buf).unwrap();
                    port.send(&buf[..]).unwrap()
                }
            }
        }
        if changed_lights {
            lights.write(&device)?;
        }
        // println!("{} {:?}", size, buf);
    }
}
