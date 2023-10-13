use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub trait P2pModule {
    // 根据账户查询Ip地址
    fn apply_p2p(&self, account: str) -> Option<SocketAddr>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pData {
    pub biz: P2pDataType,
    pub body: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum P2pDataType {
    GetIpV4Req,
    GetIpV4Resp,
    TryConnectReq,
    TryConnectResp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIpV4Req {
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIpV4Resp {
    pub account: String,
    // 符合 ip:port格式
    pub ipv4: String,
}
