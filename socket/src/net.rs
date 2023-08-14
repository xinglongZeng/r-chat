use crate::chat_protocol::{
    ChatCommand, ChatContent, ChatData, ChatFileContent, ChatTextContent, Protocol,
};
use crate::protocol_factory::{HandleProtocolFactory, HandlerProtocolData};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TcpServerState {
    INIT,
    RUNNING,
    STOPPED,
}

pub struct TcpServer {
    // addr必须是 "ip:port"的格式
    addr: String,
    factory: HandleProtocolFactory,
    state: TcpServerState,
    all_conn_cache: HashMap<SocketAddr, ProtocolCacheData>,
}

impl TcpServer {
    pub fn new(addr: String, factory: HandleProtocolFactory) -> Self {
        TcpServer {
            addr,
            factory,
            state: TcpServerState::INIT,
            all_conn_cache: Default::default(),
        }
    }

    pub fn get_state(&self) -> &TcpServerState {
        &self.state
    }

    // pub fn get_state_by_mut(&mut self)->&TcpServerState{
    //     &self.state
    // }

    pub fn start(&mut self) {
        self.state = TcpServerState::RUNNING;

        start_server_accept(self);
    }

    pub fn stop(&mut self) {
        self.state = TcpServerState::STOPPED;
    }
}

pub struct ProtocolCacheData {
    stream: TcpStream,

    data: Option<Protocol>,
}

fn start_server_accept(server: &mut TcpServer) {
    let listener = TcpListener::bind(server.addr.clone()).unwrap();

    println!("##########  TcpServer started! ###########");

    while server.state == TcpServerState::RUNNING {
        let (stream, address) = listener.accept().unwrap();

        parse_tcp_stream(stream, address, &mut server.all_conn_cache, &server.factory);
    }

    println!("##########  TcpServer stopped! ###########");
}

fn parse_tcp_stream(
    stream: TcpStream,
    address: SocketAddr,
    all_cache: &mut HashMap<SocketAddr, ProtocolCacheData>,
    factory: &HandleProtocolFactory,
) {
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

    let mut remain = cache.stream.read(&mut buf).unwrap();

    let total_len = remain.clone();

    let mut index = 0;

    let mut pkg = cache.data.as_mut().unwrap();

    let buffer = buf.to_vec();

    while remain > 0 {
        let len = fill(&mut pkg, &buffer, index.clone(), total_len.clone());

        remain -= len;

        index += len.clone();

        if pkg.completion() {
            handle_pkg(&pkg, factory);
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

fn handle_pkg(pkg: &Protocol, factory: &HandleProtocolFactory) {
    // convert bytes to struct by type
    let data_type = pkg.data_type.as_ref().unwrap()[0].clone();
    let command = ChatCommand::to_self(data_type);
    let handler = factory.get_handler(&command);
    handler.handle(pkg.data.as_ref().unwrap());
}

// 连接到指定地址
pub fn connect<A: ToSocketAddrs>(address: A) -> std::io::Result<TcpStream> {
    TcpStream::connect(address)
}

pub fn send_msg(stream: &mut TcpStream, data: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let _write_len = stream.write_all(data.as_slice())?;
    stream.flush()?;
    Ok(())
}

pub fn create_init_factory() -> HandleProtocolFactory {
    let factory = HandleProtocolFactory::new();
    factory
}

/////////////////  todo: test
pub fn get_chat_vec() -> Vec<u8> {
    let text = ChatContent::Text(ChatTextContent {
        text: "hello".to_string(),
    });

    let f = ChatContent::File(ChatFileContent {
        file_name: "test.txt".to_string(),
        data: Some(vec![1, 1, 1, 1]),
        url: None,
    });

    let v = vec![text, f];
    let c = ChatData {
        from_account: "1".to_string(),
        to_account: "2".to_string(),
        contents: v,
    };

    bincode::serialize(&c).unwrap()
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
