use crate::biz_module::ChatData;
use crate::{Module, ModuleEngine, ModuleNameEnum};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Error;
use std::net::SocketAddr;
use std::rc::Weak;
use std::sync::RwLock;

//聊天模块
trait ChatModule: Module {
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

struct DefaultClientChatModule<T: Any + Module + Sized + 'static> {
    share: Weak<ModuleEngine<T>>,
    // 读写锁，存储对方账户与socket的映射关系
    connected_accounts: RwLock<HashMap<String, SocketAddr>>,
}

impl<T: Any + Module + Sized> Module for DefaultClientChatModule<T> {
    fn get_module_name() -> ModuleNameEnum {
        ModuleNameEnum::Chat
    }
}

impl<T: Any + Module + Sized + 'static> ChatModule for DefaultClientChatModule<T> {
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
