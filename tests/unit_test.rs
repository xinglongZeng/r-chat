use std::env;
use async_trait::async_trait;
use log::info;
use r_chat::chat_protocol::{calculate_len_by_data, ChatCommand, ChatContent, ChatData, ChatFileContent, ChatTextContent, Protocol};
use r_chat::net;
use r_chat::net::{create_factory, get_chat_vec};
use r_chat::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};

#[tokio::test]
async fn test_start_tcp_socket(){

    let factory = create_factory();

    let addr="localhost:9999";

    net::start_tcp_server(addr,&factory)
        .await
        .unwrap();


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

