use crate::{Module, ModuleEngine};
use std::net::SocketAddr;
use std::sync::Arc;

trait P2pModule<T: Module + Sized>: Module {
    // 根据账户查询Ip地址
    fn apply_p2p(share: Arc<ModuleEngine<T>>, account: str) -> Option<SocketAddr>;
}
