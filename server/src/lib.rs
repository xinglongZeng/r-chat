use common::TcpSocketConfig;
use log::info;
use socket::chat_protocol::{ChatCommand, ChatData};
use socket::net::TcpServer;
use socket::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use std::thread;

pub fn start_server() {
    // 开启用户信息的web服务
    let userinfo_web =
        thread::spawn(|| userinfo_web::start_webserver_userinfo().expect("webserver start fail!"));

    // 开启socket服务
    let socket = thread::spawn(|| {});
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
    factory.registry_handler(ChatCommand::Chat, Box::new(ServerChatHandler {}));
    factory
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
