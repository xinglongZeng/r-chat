use common::TcpSocketConfig;
use log::{error, info};
use socket::chat_protocol::P2pDataType::*;
use socket::chat_protocol::{ChatCommand, ChatData, GetIpV4Req, P2pData};
use socket::net::TcpServer;
use socket::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use std::sync::{Arc, Mutex};
use std::{env, thread};
use userinfo_web::sea_orm::Database;
use userinfo_web::userinfo_dao::Dao;
use userinfo_web::userinfo_service::Service;

pub fn start_server() {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // establish connection to database.   建立与数据的链接
    let conn = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { Database::connect(&db_url).await.unwrap() });

    let dao = Dao { db: conn };

    let service = Arc::new(Service { dao });

    let service_cp = service.clone();

    // 开启用户信息的web服务
    let userinfo_web_task = thread::spawn(|| {
        userinfo_web::start_webserver_userinfo(service_cp).expect("webserver start fail!")
    });

    // 开启socket服务
    let socket_task = thread::spawn(|| start_socket());

    userinfo_web_task
        .join()
        .expect("userinfo_web_task start fail!");
    socket_task.join().expect("socket_task fail!");
}

// 开启socket服务
fn start_socket() {
    let factory = create_factory();

    let config = TcpSocketConfig::init_from_env();

    let mut server = TcpServer::new(config.get_url(), factory);

    server.start();
}

// 创建HandleProtocolFactory, 实际里面填充解析socket协议的handler
fn create_factory() -> HandleProtocolFactory {
    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(ChatCommand::Login, Box::new(ServerLoginHandler {}));
    factory.registry_handler(ChatCommand::Chat, Box::new(ServerChatHandler {}));
    factory.registry_handler(ChatCommand::P2p, Box::new(ServiceP2pHandler {}));
    factory
}
pub struct ServerLoginHandler {}

impl HandlerProtocolData for ServerLoginHandler {
    fn handle(&self, a: &Vec<u8>) {
        // todo : 处理登录
        todo!()
    }
}

pub struct ServerChatHandler {}

impl HandlerProtocolData for ServerChatHandler {
    // note: this function could do  what you want  it
    // for example ,you could record this ChatData in db. but this time ,just print it by info!.
    fn handle(&self, a: &Vec<u8>) {
        let req: ChatData = bincode::deserialize(a).unwrap();
        info!("OverrideChatHandler received data :{:?}  ", req);
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
    fn handle(&self, a: &Vec<u8>) {
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
