use crate::login_module::TestLoginActor;
use crate::socket_module::Protocol;
use crate::CommonModule;
use enum_index::IndexEnum;
use enum_index_derive::{EnumIndex, IndexEnum};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub struct DefaultBizModule {
    login: Option<TestLoginActor>,
}

impl DefaultBizModule {
    pub fn init(login: Option<TestLoginActor>) -> Self {
        DefaultBizModule { login }
    }
    fn handle_login(&mut self, data: Vec<u8>, address: SocketAddr) -> Option<Vec<u8>> {
        self.login
            .as_mut()
            .unwrap()
            .handle_byte_on_socket(data, address)
    }

    fn handle_chat_msg(&self, data: Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }

    fn handle_p2p(&self, data: Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }

    pub fn handle_pkg(&mut self, pkg: &Protocol, address: SocketAddr) -> Option<Vec<u8>> {
        let data_type = pkg.data_type.as_ref().unwrap()[0].clone();
        let biz_type: BizTypeEnum = BizTypeEnum::to_self(data_type);
        match biz_type {
            BizTypeEnum::Login => {
                return self.handle_login(pkg.data.as_ref().unwrap().to_owned(), address);
            }
            BizTypeEnum::Chat => {
                return self.handle_chat_msg(pkg.data.as_ref().unwrap().to_owned());
            }
            BizTypeEnum::P2p => {
                return self.handle_p2p(pkg.data.as_ref().unwrap().to_owned());
            }
        }
    }
}

#[derive(Debug, Clone, EnumIndex, IndexEnum, Hash, Serialize, Deserialize)]
pub enum BizTypeEnum {
    Login,
    Chat,
    P2p,
}

impl BizTypeEnum {
    pub fn to_data_type(self) -> Vec<u8> {
        let v = self as u8;
        vec![v]
    }

    pub fn to_self(b: u8) -> Self {
        BizTypeEnum::index_enum(b as usize).unwrap()
    }
}
