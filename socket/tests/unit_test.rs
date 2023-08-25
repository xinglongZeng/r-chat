use common::TcpSocketConfig;
use socket::chat_protocol::{calculate_len_by_data, ChatCommand, ChatData, Protocol};
use socket::net;
use socket::net::{get_chat_vec, TcpServer};
use socket::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use std::env;

#[test]
fn test_start_tcp_socket() {
    let factory = create_factory();

    let config = TcpSocketConfig::init_from_env();

    let mut server = TcpServer::new(config.get_url(), factory);

    server.start();
}

#[test]
fn test_send_msg() {
    let addr = "localhost:19999";

    dotenvy::dotenv().ok();

    let protocol_version =
        env::var("PROTOCOL_VERSION").expect("PROTOCOL_VERSION is not set in .env file");

    let version = protocol_version.as_bytes().to_vec();

    let data_type = ChatCommand::Chat.to_data_type();

    let data = get_chat_vec();

    let len = calculate_len_by_data(&data);

    let mut protocol = Protocol {
        version: Some(version),
        data_type: Some(data_type),
        data_len: Some(len),
        data: Some(data),
    };

    let mut stream = net::connect(addr).unwrap();

    net::send_msg(&mut stream, &protocol.to_vec()).unwrap();
}

// create factory for test
pub fn create_factory() -> HandleProtocolFactory {
    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(ChatCommand::Chat, Box::new(TestChatHandler {}));
    factory
}

pub struct TestChatHandler {}

impl HandlerProtocolData for TestChatHandler {
    fn handle(&self, a: &Vec<u8>) {
        let req: ChatData = bincode::deserialize(a).unwrap();
        println!("TestChatHandler received data :{:?}  ", req);
    }
}
