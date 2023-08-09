use std::env;
use actix::Actor;
use async_trait::async_trait;
use log::info;
use r_chat::chat_protocol::{calculate_len_by_data, ChatCommand, ChatContent, ChatData, ChatFileContent, ChatTextContent, Protocol};
use r_chat::net;
use r_chat::net::{create_factory, get_chat_vec, RegistryHandler, TcpServerActor, TestChatHandler};
use r_chat::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};

#[test]
 fn test_start_tcp_socket(){

    let factory = create_factory();

    let addr="localhost:9999".to_string();

    let server = TcpServerActor::new(addr,factory);

    let addr= server.start();

    let rh = RegistryHandler{
        command:ChatCommand::Chat,
        handler:Box::new(TestChatHandler{}),
    };

    // todo:
    // addr.send(rh);

}


// #[tokio::test]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_send_msg(){

    let addr="localhost:9999";

    dotenvy::dotenv().ok();

    let protocol_version =
        env::var("PROTOCOL_VERSION").expect("PROTOCOL_VERSION is not set in .env file");

    let version = protocol_version.as_bytes().to_vec();

    let data_type = ChatCommand::Chat.to_data_type();

    let data =get_chat_vec();

    let len = calculate_len_by_data(&data);

    let mut protocol = Protocol {
        version: Some(version),
        data_type: Some(data_type),
        data_len: Some(len),
        data: Some(data),
    };

    let mut stream= net::connect(addr).await.unwrap();

    net::send_msg(&mut stream, &protocol.to_vec()).await.unwrap();

}

