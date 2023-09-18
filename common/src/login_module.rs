use crate::config::ClientDefaultConfig;
use crate::CommonModule;
use actix::dev::MessageResponse;
use actix::{Actor, Addr, Context, Handler, Message};
use log::error;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use std::fs;
use std::fs::File;
use std::os::unix::prelude::FileExt;

pub trait LoginModule {
    fn handle_login_req(&self, req: LoginReqData) -> Result<LoginRespData, Error> {
        panic!("暂未实现该函数 [handle_login_req]!");
    }

    // 处理登录响应
    fn handle_login_resp(&mut self, resp: LoginRespData) {
        panic!("暂未实现该函数 [handle_login_resp]!");
    }

    // 获取当前缓存的登录信息
    fn get_login_cache_info(&self) -> Option<LoginRespData> {
        panic!("暂未实现该函数 [get_login_cache_info]!");
    }

    fn check_token_timeout(&self) -> Result<bool, Error> {
        panic!("暂未实现该函数 [check_token_timeout]!");
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

struct DefaultClientLoginModule {
    // 账户信息存储路径
    save_path: String,
    // 缓存的账户信息
    cache_account_info: Option<LoginRespData>,
}

pub struct TestLoginActor {
    client: Option<DefaultClientLoginModule>,
    server: Option<DefaultServerLoginModule>,
}

impl Actor for TestLoginActor {
    type Context = Context<Self>;
}

impl Handler<BizResult<LoginRespData>> for TestLoginActor {
    type Result = ();

    fn handle(&mut self, msg: BizResult<LoginRespData>, ctx: &mut Self::Context) -> Self::Result {
        let module = &mut self.client.unwrap();
        module.handle_login_biz_resp(msg);
    }
}

struct DefaultServerLoginModule {}

impl LoginModule for DefaultServerLoginModule {
    fn handle_login_req(&self, req: LoginReqData) -> Result<LoginRespData, Error> {
        todo!()
    }
}

impl Handler<LoginReqData> for TestLoginActor {
    type Result = LoginRespData;

    fn handle(&mut self, msg: LoginReqData, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Message)]
#[rtype(result = "LoginRespData")]
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

#[derive(Debug, Serialize, Deserialize, Message, Clone)]
#[rtype(result = "()")]
pub struct BizResult<T> {
    // 是否成功的标识
    pub is_success: bool,
    // is_success为fail时才有值
    pub msg: Option<String>,
    // 数据
    pub data: Option<T>,
}

impl<A, M> MessageResponse<A, M> for LoginRespData
where
    A: Actor,
    M: Message<Result = LoginRespData>,
{
    // 将 返回值 发送出去的逻辑
    fn handle(self, ctx: &mut A::Context, tx: Option<tokio::sync::oneshot::Sender<M::Result>>) {
        if let Some(tx) = tx {
            tx.send(self)
                .expect("MessageResponse#LoginRespData handle fail!");
        }
    }
}

fn get_testClientLoginActor_sender() -> Addr<TestLoginActor> {
    let config = &ClientDefaultConfig::init_from_env();
    let clientLoginModule = DefaultClientLoginModule {
        save_path: config.account_save_path.clone(),
        cache_account_info: None,
    };
    let actor = TestLoginActor {
        client: Some(clientLoginModule),
        server: None,
    };
    let addr = actor.start();
    return addr;
}

impl LoginModule for DefaultClientLoginModule {
    fn handle_login_resp(&mut self, resp: LoginRespData) {
        // 存储账户信息到文件
        save_account_info(&self.save_path, &resp);
        //  存储账户信息到缓存
        self.cache_account_info = Some(resp);
    }

    fn get_login_cache_info(&self) -> Option<LoginRespData> {
        match &self.cache_account_info {
            None => None,
            Some(t) => Some(t.clone()),
        }
    }
    fn check_token_timeout(&self) -> Result<bool, Error> {
        match &self.cache_account_info {
            None => {
                panic!("账户信息为空!");
            }
            Some(_) => {
                // todo: 检查token是否超时，目前token的生成规则还未定,暂时返回false
                Ok(false)
            }
        }
    }
}

impl DefaultClientLoginModule {
    fn handle_login_biz_resp(&mut self, resp: BizResult<LoginRespData>) {
        if !resp.is_success {
            error!("socket msg :{:?}", resp);
        } else {
            self.handle_login_resp(resp.data.unwrap());
        }
    }
}

fn save_account_info(path: &String, data: &LoginRespData) {
    // 1. 转换data为字节
    let mut byte_result = bincode::serialize(data).unwrap().as_slice();

    // 2. 将字节数据存储到文件中
    fs::create_dir_all(path).expect("创建account存储目录失败!");
    let file_name = format!("{}/{}", path, data.account);
    let file = File::create(file_name).unwrap();
    file.write_all_at(byte_result, 0).unwrap();
}

impl CommonModule for TestLoginActor {
    fn handle_byte_on_socket(&self, bytes: Vec<u8>) -> Option<Vec<u8>> {
        let login: BizLoginData = bincode::deserialize(&bytes).unwrap();

        // 处理登录请求
        match (login.login_type, login.data) {
            (LoginTypeEnum::Req, LoginDataEnum::ReqData(req)) => {
                if self.server.is_none() {
                    panic!("DefaultServerLoginModule is None!");
                }

                let resp = self.server.unwrap().handle_login_req(req);

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

            // 处理登录响应
            (LoginTypeEnum::Resp, LoginDataEnum::RespData(resp)) => {
                if self.client.is_none() {
                    panic!("DefaultClientLoginModule is None!");
                }

                self.client.unwrap().handle_login_biz_resp(resp);
            }

            _ => {
                panic!("不支持的登录数据类型!")
            }
        }

        None
    }
}
