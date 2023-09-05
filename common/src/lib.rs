use std::collections::HashMap;
use std::sync::Arc;

pub mod biz_module;
pub mod config;
pub mod socket_module;

pub mod storge_module;

pub mod ui_module;

pub mod chat_module;

pub mod p2p_module;

/**
模块抽象
**/
pub trait Module {
    fn get_module_name() -> ModuleNameEnum;
}

pub enum ModuleNameEnum {
    Socket,
    Biz,
    Chat,
    P2p,
    Storage,
    Ui,
}

struct ModuleEngine {
    all_module: HashMap<ModuleNameEnum, Box<dyn Module>>,
}

impl<T: Module + Sized> ModuleEngine {
    fn new(all_module: HashMap<ModuleNameEnum, Box<dyn Module>>) -> Self {
        ModuleEngine { all_module }
    }
}
