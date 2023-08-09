use std::collections::HashMap;
use std::{env, thread};
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::chat_protocol::{ChatCommand, ChatContent, ChatData, ChatFileContent, ChatTextContent, Protocol};
use crate::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use async_trait::async_trait;
use log::info;



#[derive(PartialEq, Clone, Copy,Debug)]
pub enum TcpServerState{
    INIT,
    RUNNING,
    STOPPED,
}

pub struct TcpServer  {
    // addr必须是 "ip:port"的格式
     addr: String,
     factory: HandleProtocolFactory,
     state: TcpServerState,
     all_conn_cache: HashMap<SocketAddr, ProtocolCacheData>,
}

pub struct TcpSocketConfig {
    pub tcp_host: String,
    pub tcp_port: String,
}

impl TcpSocketConfig {
    pub fn init_from_env() -> Self {
        dotenvy::dotenv().ok();

        let tcp_host = env::var("TCP_HOST").expect("TCP_HOST is not set in .env file");

        let tcp_port = env::var("TCP_PORT").expect("TCP_PORT is not set in .env file");

        TcpSocketConfig { tcp_host, tcp_port }
    }

    pub fn get_url(&self) -> String {
        format!("{}:{}", self.tcp_host, self.tcp_port)
    }


}

impl TcpServer{

    pub fn new(addr:String, factory:HandleProtocolFactory )->Self{
        TcpServer{
            addr,
            factory,
            state: TcpServerState::INIT,
            all_conn_cache: Default::default(),
        }
    }


    pub fn get_state(&self)->&TcpServerState{
        &self.state
    }

    // pub fn get_state_by_mut(&mut self)->&TcpServerState{
    //     &self.state
    // }

    pub async fn start(&mut self){

        self.state = TcpServerState::RUNNING;

        start_server_accept(self).await;

    }



    pub fn stop(&mut self){
        self.state=TcpServerState::STOPPED;
    }
}


pub struct ProtocolCacheData {
    stream: TcpStream,

    data: Option<Protocol>,
}


async fn start_server_accept(server: &mut TcpServer){

    let listener = TcpListener::bind(server.addr.clone()).await.unwrap();

    while server.state == TcpServerState::RUNNING {

        let (stream, address) = listener.accept().await.unwrap();

        parse_tcp_stream(stream, address, &mut server.all_conn_cache, &server.factory).await;

    }

}


async fn parse_tcp_stream(
    stream: TcpStream,
    address: SocketAddr,
    // server: &mut TcpServer,
    all_cache: &mut HashMap<SocketAddr, ProtocolCacheData>,
    factory: & HandleProtocolFactory,
) {

    // let mut all_cache= &mut server.all_conn_cache;

    // let factory = &server.factory;

    match all_cache.get_mut(&address) {
        Some(t) => match t.data {
            None => t.data = Some(Protocol::create_new()),
            Some(_) => {}
        },

        None => {
            let cache_data = ProtocolCacheData {
                stream,
                data: Some(Protocol::create_new()),
            };

            all_cache.insert(address, cache_data);
        }
    };

    let mut buf = [0; 128];

    let cache = all_cache.get_mut(&address).unwrap();

    let mut remain=cache.stream.read(&mut buf).await.unwrap();

    let total_len = remain.clone();

    let mut index = 0;

    let mut pkg = cache.data.as_mut().unwrap();

    let buffer = buf.to_vec();

    while remain > 0 {
        let len = fill(&mut pkg, &buffer, index.clone(), total_len.clone());

        remain -= len;

        index += len.clone();

        if pkg.completion() {
            handle_pkg(&pkg, factory).await;
        }
    }
}


fn fill(pkg: &mut Protocol, all_bytes: &Vec<u8>, mut index: usize, total_len: usize) -> usize {
    while index < total_len && !pkg.completion() {
        for field_name in Protocol::get_all_filed_name() {
            // 如果字段没有填充完成，则进行填充
            if !pkg.check_field_fill(&field_name) {
                let len = pkg.get_diff_size(&field_name);

                let bytes: Vec<u8> = all_bytes[index..index.clone() + len].to_vec();

                pkg.fill_field(&field_name, bytes);

                index += len.clone();
            }
        }
    }

    return index;
}


async fn handle_pkg(pkg: &Protocol, factory: &HandleProtocolFactory) {
    println!("{:?}", pkg);

    // convert bytes to struct by type
    let data_type = pkg.data_type.as_ref().unwrap()[0].clone();
    let command = ChatCommand::to_self(data_type);
    let handler = factory.get_handler(&command);
    handler.handle(pkg.data.as_ref().unwrap()).await;
}


// 异步连接到指定地址
pub async fn connect<A: ToSocketAddrs>(address: A) -> Result<TcpStream, Box<dyn Error>> {
    let conn = TcpStream::connect(address).await?;

    Ok(conn)
}


pub async fn send_msg(stream: &mut TcpStream, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let _write_len = stream.write_all(data.as_slice()).await?;
    stream.flush().await?;
    Ok(())
}

pub fn create_init_factory()->HandleProtocolFactory{
    let factory = HandleProtocolFactory::new();
    factory
}

/////////////////  todo: test
pub fn get_chat_vec()->Vec<u8>{

    let text = ChatContent::Text(
        ChatTextContent{ text: "hello".to_string() }
    );

    let f = ChatContent::File(
        ChatFileContent{file_name:"test.txt".to_string(),data:vec![1,1,1,1] }
    );

    let v = vec![text,f];
    let c = ChatData{
        from_account:"1".to_string(),
        to_account:"2".to_string(),
        contents:v
    };

    bincode::serialize(&c).unwrap()
}


// create factory for test
pub fn create_factory()->HandleProtocolFactory{
    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(ChatCommand::Chat, Box::new(TestChatHandler {}));
    factory
}



pub struct TestChatHandler{

}

#[async_trait]
impl HandlerProtocolData for TestChatHandler{
    async fn handle(&self, a: &Vec<u8>) {
        let req: ChatData = bincode::deserialize(a).unwrap();
        info!("TestChatHandler received data :{:?}  ", req);
    }
}
