use crate::biz_module::ChatData;
use crate::{Module, ModuleEngine};
use std::fmt::Error;
use std::sync::Arc;

//聊天模块
trait ChatModule<T: Module + Sized>: Module {
    // 发送信息到指定账户
    fn sendMsg(
        share: Arc<ModuleEngine<T>>,
        from_account: str,
        to_account: str,
        msg: str,
    ) -> Result<(), Error>;

    // 查看指定账户之间的聊天记录
    fn find_chat_history(
        share: Arc<ModuleEngine<T>>,
        account_a: str,
        account_b: str,
    ) -> Vec<ChatData>;
}
