use crate::login_module::{
    BizLoginData, LoginDataEnum, LoginModule, LoginReqData, LoginRespData, LoginTypeEnum,
};
use crate::socket_module::Protocol;
use crate::{ModuleActorEngine, ModuleEngine, ModuleNameEnum};
use actix::Message;
use enum_index::IndexEnum;
use enum_index_derive::{EnumIndex, IndexEnum};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct DefaultBizModule {
    // share: Weak<ModuleEngine>,
    share: Arc<ModuleActorEngine>,
}

impl DefaultBizModule {
    fn handle_login(&self, data: &Vec<u8>) -> Option<Vec<u8>> {}

    fn handle_chat_msg(&self, data: &Vec<u8>) {
        todo!()
    }

    fn handle_p2p(&self, data: &Vec<u8>) {
        todo!()
    }

    pub fn handle_pkg(&self, pkg: &Protocol) {
        let data_type = pkg.data_type.as_ref().unwrap()[0].clone();
        let biz_type: BizTypeEnum = BizTypeEnum::to_self(data_type);
        match biz_type {
            BizTypeEnum::Login => {
                self.handle_login(pkg.data.as_ref().unwrap());
            }
            BizTypeEnum::Chat => {
                self.handle_chat_msg(pkg.data.as_ref().unwrap());
            }
            BizTypeEnum::P2p => {
                self.handle_p2p(pkg.data.as_ref().unwrap());
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
