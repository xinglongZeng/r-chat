use crate::protocol_factory::HandlerProtocolData;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub struct DefaultLoginHandler {
    // 是否是server端的标识
    server_flg: bool,

    server: Option<Box<dyn ServerLoginModule + Send>>,

    client: Option<Box<dyn ClientLoginModule + Send>>,
}

impl DefaultLoginHandler{
    
    pub fn new(server_flg:bool ,server: Option<Box<dyn ServerLoginModule + Send>>, client: Option<Box<dyn ClientLoginModule + Send>>,)->Self{
        DefaultLoginHandler{server_flg,server,client}
    }
    
}


impl HandlerProtocolData for DefaultLoginHandler {
    fn handle(&mut self, address: SocketAddr, data: &Vec<u8>) -> Option<Vec<u8>> {
        // 反序列化为 BizLoginData
        let login: BizLoginData = bincode::deserialize(data).unwrap();

        // server端处理请求
        match (login.login_type, login.data) {
            (LoginTypeEnum::Req, LoginDataEnum::ReqData(req)) => {
                if self.server.is_none() {
                    panic!("ServerLoginModule is None!");
                }
                let resp = self.server.as_mut().unwrap().handle_login_req(req, address);

                let mut bizResult: Option<BizResult<LoginRespData>> = None;

                if resp.is_err() {
                    let t = BizResult {
                        is_success: false,
                        msg: Some(resp.err().unwrap().to_string()),
                        data: None,
                    };
                    bizResult = Some(t);
                } else {
                    let t = BizResult {
                        is_success: true,
                        msg: None,
                        data: Some(resp.unwrap()),
                    };
                    bizResult = Some(t);
                }

                return Some(bincode::serialize(&bizResult.unwrap()).unwrap());
            }

            // client端处理响应
            (LoginTypeEnum::Resp, LoginDataEnum::RespData(resp)) => {
                if self.client.is_none() {
                    panic!("ClientLoginModule is None!");
                }

                self.client.as_mut().unwrap().handle_login_biz_resp(resp);
            }

            _ => {
                panic!("不支持的login数据类型!")
            }
        }

        None
    }
}

/**
*  默认的server端处理登录请求的模块trait
**/
pub trait ServerLoginModule {
    fn handle_login_req(
        &mut self,
        req: LoginReqData,
        address: SocketAddr,
    ) -> Result<LoginRespData, String> {
        panic!("暂未实现该函数 [handle_login_req]!");
    }
}

/**
 *  默认的client端处理登录响应的模块trait
 **/
pub trait ClientLoginModule {
    fn handle_login_biz_resp(&mut self, resp: BizResult<LoginRespData>) {
        panic!("暂未实现该函数 [handle_login_biz_resp]!");
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BizLoginData {
    pub login_type: LoginTypeEnum,
    pub data: LoginDataEnum,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginDataEnum {
    ReqData(LoginReqData),
    RespData(BizResult<LoginRespData>),
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRespData {
    pub user_id: i32,
    pub account: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BizResult<T> {
    // 是否成功的标识
    pub is_success: bool,
    // is_success为fail时才有值
    pub msg: Option<String>,
    // 数据
    pub data: Option<T>,
}
