use crate::chat_module::ChatModule;
use crate::p2p_module::P2pModule;
use crate::storage_module::StorageModule;
use crate::ui_module::UiModule;
use std::any::Any;

pub mod chat_module;
pub mod chat_protocol;
pub mod config;
pub mod login_module;
pub mod p2p_module;
pub mod protocol_factory;
pub mod storage_module;
pub mod ui_module;

pub mod cli;

pub mod base;

pub mod errors_define;

pub use structopt;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ModuleNameEnum {
    Socket,
    Biz,
    Chat,
    P2p,
    Storage,
    Ui,
    Login,
}
