use serde::{Deserialize, Serialize};

use shared::audio_settings::AudioSettings;
use shared::breakpoints::Breakpoint;
use shared::input::Keybindings;

use crate::emulator::EmuSettings;
use crate::settings::Mode;

// paths are OS specific paths to rom directories/files mixup
// each of these paths should be handed to Rom::find_roms for recursive dir traversal and rom retrieval
// if no roms.conf file is found (use default directories later, for now project root path)
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename = "roms")]
pub struct RomConfig {
    pub(crate) paths: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename = "debug")]
pub struct DbgConfig {
    pub breaks: Vec<Breakpoint>,
    #[serde(default)]
    pub and: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub roms: RomConfig,
    #[serde(default)]
    pub debug: DbgConfig,
    #[serde(default)]
    pub keys: Keybindings,
    #[serde(default)]
    pub sound_device: apu::SoundConfig,
    #[serde(default)]
    pub audio_settings: AudioSettings,
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub emu: EmuSettings,
    #[serde(default)]
    pub bios: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        serde_any::from_file("gbmu.ron").unwrap_or_else(|_| Default::default())
    }
}
