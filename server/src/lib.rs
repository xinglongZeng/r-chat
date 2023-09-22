use common::biz_module::DefaultBizModule;
use common::config::TcpSocketConfig;
use common::login_module::{LoginModule, LoginReqData, LoginRespData, TestLoginActor};
use common::socket_module::{DefaultSocketModule, SocketModule};
use log::{error, info, warn};
use socket::chat_protocol::P2pDataType::*;
use socket::chat_protocol::{ChatCommand, ChatData, GetIpV4Req, P2pData};
use socket::net::TcpServer;
use socket::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use std::collections::HashMap;
use std::fmt::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use std::{env, thread};
use userinfo_web::sea_orm::Database;
use userinfo_web::userinfo_dao::Dao;
use userinfo_web::userinfo_service::Service;

pub fn start_server_new() {
    let user_info_service = init_user_info_service();
    let arc_user_service = Arc::new(user_info_service);
    let mut socket_module = init_socket_module(arc_user_service);
    // todo
    socket_module.start();
}

fn init_socket_module(user_service: Arc<Service>) -> DefaultSocketModule {
    let d_s_login = DefaultServerLoginModule::init(user_service);
    let login_module = TestLoginActor::init(None, Some(Box::new(d_s_login)));
    let share = DefaultBizModule::init(Some(login_module));
    let socket_module = DefaultSocketModule::init(share);
    socket_module
}

pub struct DefaultServerLoginModule {
    user_service: Arc<Service>,
    login_cache: HashMap<String, SocketAddr>,
}

impl DefaultServerLoginModule {
    fn init(user_service: Arc<Service>) -> Self {
        DefaultServerLoginModule {
            user_service,
            login_cache: Default::default(),
        }
    }

    fn update_cache(&mut self, address: SocketAddr, account: String) {
        self.login_cache.insert(account, address).unwrap();
    }

    fn find_address_by_account_from_cache(&self, account: &String) -> SocketAddr {
        self.login_cache.get(account).unwrap().clone()
    }
}

impl LoginModule for DefaultServerLoginModule {
    fn handle_login_req(
        &mut self,
        req: LoginReqData,
        address: SocketAddr,
    ) -> Result<LoginRespData, String> {
        let user_service_ref = Arc::clone(&self.user_service);

        // get account info
        let account_info = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(user_service_ref.find_by_account_and_pwd(&req));

        let result = match account_info {
            Ok(t) => {
                // todo: 生成token
                let token = "token".to_string();
                if t.is_some() {
                    let model = t.unwrap();
                    let resp = LoginRespData {
                        user_id: model.id,
                        account: model.name,
                        token,
                    };
                    // insert cache
                    self.login_cache.insert(model.name.clone(), address);
                    Ok(resp)
                } else {
                    Err("login fail : account of password err !".to_string())
                }
            }

            Err(e) => Err(e),
        };

        return result;
    }
}

pub fn start_server() {
    let service = Arc::new(init_user_info_service);

    let service_cp = service.clone();

    // 开启用户信息的web服务
    let userinfo_web_task = thread::spawn(|| {
        userinfo_web::start_webserver_userinfo(service_cp).expect("webserver start fail!")
    });

    let service_cp2 = service.clone();

    // 开启socket服务
    let socket_task = thread::spawn(|| start_socket(service_cp2));

    userinfo_web_task
        .join()
        .expect("userinfo_web_task start fail!");
    socket_task.join().expect("socket_task fail!");
}

fn init_user_info_service() -> Service {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // establish connection to database.   建立与数据的链接
    let conn = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { Database::connect(&db_url).await.unwrap() });

    let dao = Dao { db: conn };
    Service { dao }
}

// 开启socket服务
fn start_socket(user_service: Arc<Service>) {
    let factory = create_factory(user_service);

    let config = TcpSocketConfig::init_from_env();

    let mut server = TcpServer::new(config.get_url(), factory);

    server.start();
}

// 创建HandleProtocolFactory, 实际里面填充解析socket协议的handler
fn create_factory(user_service: Arc<Service>) -> HandleProtocolFactory {
    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(
        ChatCommand::Login_req,
        Box::new(ServerLoginHandler {
            user_service,
            login_record: Arc::new(Default::default()),
        }),
    );
    factory.registry_handler(ChatCommand::Chat, Box::new(ServerChatHandler {}));
    factory.registry_handler(ChatCommand::P2p, Box::new(ServiceP2pHandler {}));
    factory
}
pub struct ServerLoginHandler {
    user_service: Arc<Service>,
    login_record: Arc<RwLock<HashMap<i32, SocketAddr>>>,
}

impl ServerLoginHandler {
    fn create_login_record(&self, uid: i32, address: SocketAddr) -> bool {
        let read_lock = self.login_record.try_read();
        if read_lock.is_err() {
            warn!("[ServerLoginHandler] 获取读锁失败！uid:{}", uid);
            return false;
        }
        read_lock.unwrap().insert(uid, address);
        warn!(
            "[ServerLoginHandler]-[create_login_record]-[success]! uid:{}, address:{}",
            uid, address
        );
        true
    }
}

impl HandlerProtocolData for ServerLoginHandler {
    fn handle(&self, address: SocketAddr, a: &Vec<u8>) -> Option<Vec<u8>> {
        let req: LoginReqData = bincode::deserialize(a).unwrap();

        let op = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                self.user_service
                    .find_by_account_and_pwd(&req)
                    .await
                    .unwrap()
            });

        if op.is_none() {
            return None;
        }

        let t = op.unwrap();
        let data = LoginRespData {
            user_id: t.id,
            account: t.name,
            // todo: 生成token
            token: "1234".to_string(),
        };

        //  创建登录记录
        if !self.create_login_record(t.id, address) {
            return None;
        }

        let ser = bincode::serialize(&data);
        match ser {
            Ok(v) => Some(v),
            Err(e) => {
                warn!("[ServerLoginHandler] :  can't serialize LoginRespData!");
                None
            }
        }
    }
}

pub struct ServerChatHandler {}

impl HandlerProtocolData for ServerChatHandler {
    // note: this function could do  what you want  it
    // for example ,you could record this ChatData in db. but this time ,just print it by info!.
    fn handle(&self, address: SocketAddr, a: &Vec<u8>) -> Option<Vec<u8>> {
        let req: ChatData = bincode::deserialize(a).unwrap();
        info!("OverrideChatHandler received data :{:?}  ", req);
        None
    }
}

// handle msg "p2p" on server side
pub struct ServiceP2pHandler {}

impl ServiceP2pHandler {
    fn handleGetIpV4Req(&self, a: &Vec<u8>) {
        let req: GetIpV4Req = bincode::deserialize(a).unwrap();

        // todo: 读取db或缓存，获取指定账户的ip地址，然后封装成GetIpV4Resp，再通过socket返回
    }

    fn handleTryConnectReq(&self, a: &Vec<u8>) {
        todo!()
    }
}

impl HandlerProtocolData for ServiceP2pHandler {
    fn handle(&self, address: SocketAddr, a: &Vec<u8>) {
        // todo: 获取biz类型
        let param: P2pData = bincode::deserialize(a).unwrap();
        match param.biz {
            GetIpV4Req => {
                self.handleGetIpV4Req(a);
            }
            TrtConnectReq => {
                self.handleTryConnectReq(a);
            }
            _ => {
                error!("暂不支持的biz:{:?}", param.biz);
            }
        }
    }
}
