use crate::login_module::LoginModule;
use crate::socket_module::Protocol;
use crate::{Module, ModuleEngine, ModuleNameEnum};
use enum_index::IndexEnum;
use enum_index_derive::{EnumIndex, IndexEnum};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::rc::Weak;

pub struct DefaultBizModule<T: Any + Module + Sized + 'static> {
    share: Weak<ModuleEngine<T>>,
}

impl<T: Any + Module + Sized + 'static> Module for DefaultBizModule<T> {
    fn get_module_name() -> ModuleNameEnum {
        ModuleNameEnum::Biz
    }
}

impl<T: Any + Module + Sized + 'static> DefaultBizModule<T> {
    fn handle_login(&self, data: &Vec<u8>) {
        let login_data: BizLoginData = bincode::deserialize(data).unwrap();

        // 通过
        let any_module = self
            .share
            .upgrade()
            .unwrap()
            .all_module
            .get(&ModuleNameEnum::Login)
            .unwrap() as &dyn Any;

        let login_module = any_module.downcast_ref::<dyn LoginModule>().unwrap();

        match login_data.login_type {
            LoginTypeEnum::Req => {
                //  处理登录请求
                login_module.handle_login_req(login_data.data);
            }
            LoginTypeEnum::Resp => {
                // 登录登入响应
                login_module.handle_login_resp(login_data.data);
            }
        }
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub struct BizLoginData {
    login_type: LoginTypeEnum,
    data: LoginDataEnum,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginDataEnum {
    ReqData(LoginReqData),
    RespData(LoginRespData),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginTypeEnum {
    Req,
    Resp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReqData {
    pub account: String,
    pub pwd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRespData {
    pub user_id: i32,
    pub account: String,
    pub token: String,
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
    TrtConnectReq,
    TrtConnectResp,
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
