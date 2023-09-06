use std::any::Any;
use std::collections::HashMap;

pub mod biz_module;
pub mod chat_module;
pub mod config;
pub mod login_module;
pub mod p2p_module;
pub mod socket_module;
pub mod storge_module;
pub mod ui_module;

/**
模块抽象
**/
pub trait Module {
    fn get_module_name() -> ModuleNameEnum;
}

#[derive(Debug)]
pub enum ModuleNameEnum {
    Socket,
    Biz,
    Chat,
    P2p,
    Storage,
    Ui,
    Login,
}

struct ModuleEngine<T: Any + Module+Sized> {
    all_module: HashMap<ModuleNameEnum, Box<T>>,
}

impl<T: Any + Module+Sized> ModuleEngine<T> {
    
    fn new(all_module: HashMap<ModuleNameEnum, Box<T>>) -> Self {
        ModuleEngine { all_module }
    }

    fn find_module(&self, name: ModuleNameEnum) -> &Box<T> {
        let all_module = &self.all_module;
        let op = all_module.get(&name);
        match op {
            None => {
                panic!("[ModuleEngine] can not find module:{:?}", name);
            }
            Some(t) => {
                return t;
            }
        }
    }
}
