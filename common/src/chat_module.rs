use crate::{ModuleEngine, ModuleNameEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Error;
use std::net::SocketAddr;
use std::rc::Weak;
use std::sync::RwLock;

//聊天模块
pub trait ChatModule {
    // 发送信息到指定账户
    fn sendMsg(&self, from_account: String, to_account: String, msg: String) -> Result<(), Error>;

    // 发送文件到指定账户
    fn sendFile(
        &self,
        from_account: String,
        to_account: String,
        file_nmae: String,
        file_path: String,
    ) -> Result<(), Error>;

    // 查看指定账户之间的聊天记录
    fn find_chat_history(&self, account_a: String, account_b: String) -> Option<Vec<ChatData>>;
}

struct DefaultClientChatModule {
    share: Weak<ModuleEngine>,
    // 读写锁，存储对方账户与socket的映射关系
    connected_accounts: RwLock<HashMap<String, SocketAddr>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatData {
    pub from_account: String,
    pub to_account: String,
    pub contents: Vec<ChatContent>,
    // todo:暂时用u32来表示时间错
    pub time: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatContent {
    Text(ChatTextContent),
    File(ChatFileContent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatTextContent {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatFileContent {
    pub file_name: String,
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
}

impl ChatModule for DefaultClientChatModule {
    fn sendMsg(&self, from_account: String, to_account: String, msg: String) -> Result<(), Error> {
        todo!()
        // 1. 根据 to_account检查是否有对应的socket链接
        // 2. 如果有，则使用该链接发送消息
    }

    fn sendFile(
        &self,
        from_account: String,
        to_account: String,
        file_nmae: String,
        file_path: String,
    ) -> Result<(), Error> {
        todo!()
    }

    fn find_chat_history(&self, account_a: String, account_b: String) -> Option<Vec<ChatData>> {
        todo!()
    }
}
