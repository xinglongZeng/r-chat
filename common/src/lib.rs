use crate::biz_module::DefaultBizModule;
use crate::chat_module::ChatModule;
use crate::login_module::{LoginModule, TestLoginActor};
use crate::p2p_module::P2pModule;
use crate::socket_module::SocketModule;
use crate::storage_module::StorageModule;
use crate::ui_module::UiModule;
use actix::{Actor, Addr, Context};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub mod biz_module;
pub mod chat_module;
pub mod config;
pub mod login_module;
pub mod p2p_module;
pub mod socket_module;
pub mod storage_module;
pub mod ui_module;

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

pub trait CommonModule {
    fn handle_byte_on_socket(&mut self, bytes: Vec<u8>) -> Option<Vec<u8>>;
}
