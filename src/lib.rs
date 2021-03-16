#![feature(concat_idents)]
#![feature(proc_macro_hygiene)]
#![feature(str_strip)]
mod config;
use config::*;

use acmd;
use skyline::hooks::{getRegionAddress, Region};
use skyline::nn::ro::LookupSymbol;
use skyline::{hook, install_hook};
use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::lua2cpp::L2CFighterCommon;
use std::convert::TryInto;
use toml::Value;

pub static mut VICTORY_COLOR_INDEX: usize = 0;
pub static mut ENTRY_ID: usize = 0;
pub static mut VICTOR: usize = 0;

pub static mut FIGHTER_MANAGER_ADDR: usize = 0;

static mut MUSIC_OFFSET: usize = 0x3451f30; // default = 8.1.0 offset

static MUSIC_SEARCH_CODE: &[u8] = &[
    0xfc, 0x6f, 0xba, 0xa9, 0xfa, 0x67, 0x01, 0xa9, 0xf8, 0x5f, 0x02, 0xa9, 0xf6, 0x57, 0x03, 0xa9,
    0xf4, 0x4f, 0x04, 0xa9, 0xfd, 0x7b, 0x05, 0xa9, 0xfd, 0x43, 0x01, 0x91, 0xff, 0xc3, 0x1b, 0xd1,
    0xe8, 0x63, 0x05, 0x91,
];

// Use this for general per-frame fighter-level hooks
pub fn once_per_fighter_frame(fighter: &mut L2CFighterCommon) {
    unsafe {
        let lua_state = fighter.lua_state_agent;
        let module_accessor = smash::app::sv_system::battle_object_module_accessor(lua_state);
        ENTRY_ID =
            WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        let fighter_manager = *(FIGHTER_MANAGER_ADDR as *mut *mut smash::app::FighterManager);
        VICTOR = FighterManager::get_top_rank_player(fighter_manager, 0) as usize;
        if ENTRY_ID == VICTOR {
            VICTORY_COLOR_INDEX =
                WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR)
                    .try_into()
                    .unwrap();
        }
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[skyline::hook(offset = MUSIC_OFFSET)]
pub fn music_function_replace(
    param_1: *mut u64,
    param_2: i64,
    nus3bank_hash: u64,
    nus3audio_hash: *const u64,
    mut nus3audio_index: usize,
) {
    unsafe {
        if VICTORY_CONFIG
            .lock()
            .unwrap()
            .entries
            .contains_key(&*nus3audio_hash)
        {
            nus3audio_index = *VICTORY_CONFIG
                .lock()
                .unwrap()
                .entries
                .get(&*nus3audio_hash)
                .unwrap()
                .id_color
                .get(&VICTORY_COLOR_INDEX)
                .unwrap_or(&0);
            VICTORY_COLOR_INDEX = 0;
        }
    }
    original!()(
        param_1,
        param_2,
        nus3bank_hash,
        nus3audio_hash,
        nus3audio_index,
    );
}

#[skyline::main(name = "one_slot_victory")]
pub fn main() {
    unsafe {
        LookupSymbol(
            &mut FIGHTER_MANAGER_ADDR,
            "_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E\u{0}"
                .as_bytes()
                .as_ptr(),
        );
    }
    lazy_static::initialize(&VICTORY_CONFIG);
    read_from_rom_path();

    match std::fs::read_to_string("sd:/atmosphere/contents/01006A800016E000/romfs/arcropolis.toml")
    {
        Ok(content) => match content.parse::<Value>().unwrap()["paths"]["umm"].as_str() {
            Some(res) => {
                read_from_umm_path(std::path::Path::new(&res.to_string()));
                println!("[One Slot Victory::main] Finished reading UMM path!");
            }
            None => println!("[One Slot Victory::main] Failed parsing ARCropolis config file!"),
        },
        Err(_) => println!("[One Slot Victory::main] 😢"),
    };

    unsafe {
        let text_ptr = getRegionAddress(Region::Text) as *const u8;
        let text_size = (getRegionAddress(Region::Rodata) as usize) - (text_ptr as usize);
        let text = std::slice::from_raw_parts(text_ptr, text_size);
        if let Some(offset) = find_subsequence(text, MUSIC_SEARCH_CODE) {
            MUSIC_OFFSET = offset;
            println!("Offset Found! Offset: {:#x}", MUSIC_OFFSET);
        } else {
            println!("Error: no offset found. Defaulting to 8.1.0 offset. This likely won't work.");
        }
    }

    acmd::add_custom_hooks!(once_per_fighter_frame);
    install_hook!(music_function_replace);
}
