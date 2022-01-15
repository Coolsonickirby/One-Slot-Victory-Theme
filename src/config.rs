use std::sync::Mutex;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use toml::Value;

lazy_static::lazy_static! {
    pub static ref VICTORY_CONFIG: Mutex<VictoryConfig> = Mutex::new(VictoryConfig::new());
}

pub struct VictorEntry {
    pub id_color: HashMap<usize, usize>,
}

pub struct VictoryConfig {
    pub entries: HashMap<u64, VictorEntry>,
}

impl VictorEntry {
    pub fn new() -> Self {
        Self {
            id_color: HashMap::new(),
        }
    }
}

impl VictoryConfig {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

pub fn read_from_umm_path(path: &Path) {
    match fs::read_dir(&path) {
        Ok(res) => {
            for entry in res {
                let entry = entry.unwrap();

                let mut entry_path = path.to_path_buf();
                entry_path.push(entry.path());

                // Ignore anything that starts with a period
                if entry_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(".")
                {
                    continue;
                }

                entry_path.push("victory.toml");

                if fs::metadata(&entry_path).is_ok() {
                    match fs::read_to_string(entry_path) {
                        Ok(content) => {
                            add_to_config(content);
                        }
                        Err(_) => {}
                    };
                }
            }
        }
        Err(_) => println!(
            "[One Slot Victory::read_from_umm_path] Path {} does not exist!",
            path.display()
        ),
    }
}

pub fn read_from_arc_path(mut path: PathBuf) {
    path.push("victory.toml");
    match fs::read_to_string(&path){
        Ok(res) => {
            add_to_config(res)
        }
        Err(_) => {println!("[One Slot Victory::read_from_rom_path] Failed to read {}", path.display())}
    }
}

fn add_to_config(content: String) {
    let out = content.parse::<Value>().unwrap();

    for item in out.as_table() {
        for (key, value) in item {
            let hash = smash::hash40(&format!("stream:/sound/bgm/{}.nus3audio", key));

            if !VICTORY_CONFIG.lock().unwrap().entries.contains_key(&hash) {
                VICTORY_CONFIG
                    .lock()
                    .unwrap()
                    .entries
                    .insert(hash, VictorEntry::new());
            }

            for val in value.as_table() {
                for (k, v) in val {
                    VICTORY_CONFIG
                        .lock()
                        .unwrap()
                        .entries
                        .get_mut(&hash)
                        .unwrap()
                        .id_color
                        .insert(
                            k.as_str().replace("costume_", "").replace("c", "").parse::<usize>().unwrap(),
                            v.as_integer().unwrap() as usize,
                        );
                }
            }
        }
    }
}
