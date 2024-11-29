#![feature(ptr_sub_ptr)]

use unity::prelude::*;
use skyline::install_hook;
use engage::gamedata::item::*;
use cobapi::{Event, SystemEvent};
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gamedata::*,
    gamedata::item::UnitItem,
    gamedata::unit::Unit
};
use skyline::patching::Patch;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;


pub const EMPTY: &str = "";
pub const ESID_KEY: &str = "G_ESID_TOGGLE";
static mut ESID_LIST: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub struct EsidMod;
impl ConfigBasicMenuItemSwitchMethods for EsidMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){ GameVariableManager::make_entry(ESID_KEY, 0);}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let toggle = GameVariableManager::get_bool(ESID_KEY);
        let result = ConfigBasicMenuItem::change_key_value_b(toggle);
        if toggle != result {
            GameVariableManager::set_bool(ESID_KEY, result);
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            patch(result);
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let sid_mode =  GameVariableManager::get_bool(ESID_KEY);
        if sid_mode 
            {this.help_text = "A unit's unique weapon can be equiped by anyone.".into(); }
        else
            {this.help_text = "A unit's unique weapon can only be equiped by that unit.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        let sid_mode =  GameVariableManager::get_bool(ESID_KEY);
        if sid_mode 
            { this.command_text = "On".into(); }
        else
            { this.command_text = "Off".into(); }
    }
}

#[no_mangle]
extern "C" fn esid_Toggle() -> &'static mut ConfigBasicMenuItem {
    ConfigBasicMenuItem::new_switch::<EsidMod>("No Unique Weapons")
}

extern "C" fn create_settings(event: &Event<SystemEvent>) {
    unsafe {

        if let Event::Args(ev) = event {
            match ev {
                SystemEvent::ProcInstJump {proc, label } => {
                    if proc.hashcode == -988690862 && *label == 0 {
                        let mut bank_list = ESID_LIST.lock().unwrap();
                        let item_list = ItemData::get_list().unwrap();
                        if bank_list.len() == 0
                        {
                            for x in 0..item_list.len()
                            {
                                let item = &item_list[x];
                                bank_list.push(Il2CppString::to_string(get_equip(item, None)));
                            }
                        }
                        sleep(Duration::from_secs(5));
                    }
                }
                _ => {},
            }
        }
        // else {  println!("We received a missing event, and we don't care!"); }
    }
}

pub fn create_variables() {
    GameVariableManager::make_entry(ESID_KEY, 0);
}

#[skyline::hook(offset = 0x2281a80)]
pub fn load_settings1(this: u64, stream: u64, method_info: OptionalMethod) -> bool {
    let value: bool = call_original!(this, stream, method_info);
    unsafe {

        if value {
            create_variables();
        }

        patch(GameVariableManager::get_bool(ESID_KEY));
    }

    return value;
}

#[unity::from_offset("App", "ItemData", "set_EquipCondition")]
pub fn set_equip(this: &ItemData, value : &'static Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "ItemData", "get_EquipCondition")]
pub fn get_equip(this: &ItemData, method_info: OptionalMethod) -> &'static Il2CppString;

pub fn patch(result: bool){
    unsafe {
        let bank_list = ESID_LIST.lock().unwrap();
        let item_list = ItemData::get_list_mut().unwrap();
        if bank_list.len() > 0
        {
            if item_list.len() != bank_list.len()
            {
                skyline::error::show_error(
                    69,
                    "Custom plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
                    &bank_list.len().to_string()
                );
            }

            for x in 0..item_list.len() {
                let item = &mut item_list[x];
                if result
                {
                    set_equip(item, "".into(), None);
                }
                else
                {
                    let its: &'static Il2CppString = Il2CppString::new_static(bank_list[x].clone());
                    set_equip(item, its, None);
                }
            }
        }
    }
}

#[skyline::main(name = "NoUniqueWeapons")]
pub fn main() {
    println!("No Unique Weapons plugin loaded");

    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        // Some magic thing to turn what was provided to the panic into a string. Don't mind it too much.
        // The message will be stored in the msg variable for you to use.
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };

        // This creates a new String with a message of your choice, writing the location of the panic and its message inside of it.
        // Note the \0 at the end. This is needed because show_error is a C function and expects a C string.
        // This is actually just a result of bad old code and shouldn't be necessary most of the time.
        let err_msg = format!(
            "Custom plugin has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        // We call the native Error dialog of the Nintendo Switch with this convenient method.
        // The error code is set to 69 because we do need a value, while the first message displays in the popup and the second shows up when pressing Details.
        skyline::error::show_error(
            69,
            "Custom plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));

    cobapi::register_system_event_handler(create_settings);
    cobapi::install_game_setting(esid_Toggle);
    skyline::install_hooks!(load_settings1);

}