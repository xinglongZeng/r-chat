use common::base::TcpServer;
use common::chat_module::ChatData;
use common::chat_protocol::ChatCommand;
use common::config::TcpSocketConfig;
use common::login_module::{DefaultLoginHandler, LoginReqData, LoginRespData, ServerLoginModule};
use common::p2p_module::{GetIpV4Req, P2pData};
use common::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use env_logger::Env;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::{env, thread};
use userinfo_web::sea_orm::Database;
use userinfo_web::userinfo_dao::Dao;
use userinfo_web::userinfo_service::Service;

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

impl ServerLoginModule for DefaultServerLoginModule {
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
                        user_id: model.id.clone(),
                        account: model.name.clone(),
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
    // get env vars   读取.env文件中的变量，相当于读取配置文件
    dotenvy::dotenv().ok();

    // set logger level to debug
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // start trace info collect.  开启堆栈信息收集
    // tracing_subscriber::fmt::init();

    let service = Arc::new(init_user_info_service());

    let service_cp = Arc::clone(&service);

    // 开启用户信息的web服务
    let userinfo_web_task = thread::spawn(|| {
        userinfo_web::start_webserver_userinfo(service_cp).expect("webserver start fail!")
    });

    let service_cp2 = Arc::clone(&service);

    // 开启socket服务
    let socket_task = thread::spawn(|| start_socket(service_cp2));

    userinfo_web_task
        .join()
        .expect("userinfo_web_task start fail!");

    socket_task.join().expect("socket_task fail!");
}

fn init_user_info_service() -> Service {
    // get env vars   读取.env文件中的变量，相当于读取配置文件
    dotenvy::dotenv().ok();

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

    let config = TcpSocketConfig::init_from_env("SERVER_TCP_HOST", "SERVER_TCP_PORT");

    let mut server = TcpServer::new(config.get_url(), factory);

    server.start();
}

// 创建HandleProtocolFactory, 实际里面填充解析socket协议的handler
fn create_factory(user_service: Arc<Service>) -> HandleProtocolFactory {
    // login handler
    let login_handler = create_default_server_login_handler(user_service);
    // todo: chat handler
    let chat_handler = Box::new(ServerChatHandler {});
    // todo: p2p handler
    let p2p_handler = Box::new(ServiceP2pHandler {});

    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(ChatCommand::Login, login_handler);
    factory.registry_handler(ChatCommand::Chat, chat_handler);
    factory.registry_handler(ChatCommand::P2p, p2p_handler);
    factory
}

fn create_default_server_login_handler(user_service: Arc<Service>) -> Box<DefaultLoginHandler> {
    let server = DefaultServerLoginModule::init(user_service);
    Box::new(DefaultLoginHandler::new(true, Some(Box::new(server)), None))
}

pub struct ServerChatHandler {}

impl HandlerProtocolData for ServerChatHandler {
    // note: this function could do  what you want  it
    // for example ,you could record this ChatData in db. but this time ,just print it by info!.
    fn handle(&mut self, address: SocketAddr, a: &Vec<u8>) -> Option<Vec<u8>> {
        let req: ChatData = bincode::deserialize(a).unwrap();
        info!("OverrideChatHandler received data :{:?}  ", req);
        None
    }
}

// handle msg "p2p" on server side
pub struct ServiceP2pHandler {}

impl ServiceP2pHandler {
    fn handleGetIpV4Req(&self, a: &Vec<u8>) -> Option<Vec<u8>> {
        let req: GetIpV4Req = bincode::deserialize(a).unwrap();

        // todo: 读取db或缓存，获取指定账户的ip地址，然后封装成GetIpV4Resp，再通过socket返回
        None
    }

    fn handleTryConnectReq(&self, a: &Vec<u8>) -> Option<Vec<u8>> {
        todo!()
    }
}

impl HandlerProtocolData for ServiceP2pHandler {
    fn handle(&mut self, address: SocketAddr, a: &Vec<u8>) -> Option<Vec<u8>> {
        // todo: 获取biz类型
        let param: P2pData = bincode::deserialize(a).unwrap();
        match param.biz {
            common::p2p_module::P2pDataType::GetIpV4Req => {
                return self.handleGetIpV4Req(a);
            }
            common::p2p_module::P2pDataType::TryConnectReq => {
                return self.handleTryConnectReq(a);
            }
            _ => {
                error!("暂不支持的biz:{:?}", param.biz);
                return None;
            }
        }
    }
}
