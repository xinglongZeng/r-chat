use crate::chat_module::{ChatContent, ChatData, ChatFileContent, ChatTextContent};
use crate::chat_protocol::Protocol;
use crate::protocol_factory::HandleProtocolFactory;
use enum_index::{EnumIndex, IndexEnum};
use enum_index_derive::{EnumIndex, IndexEnum};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, EnumIndex, IndexEnum, Hash, Serialize, Deserialize)]
pub enum RchatCommand {
    Login,
    Chat,
    P2p,
    Start,
}

impl PartialEq<Self> for RchatCommand {
    fn eq(&self, other: &Self) -> bool {
        self.enum_index() == other.enum_index()
    }
}

impl Eq for RchatCommand {}

impl RchatCommand {
    pub fn from_string(command: &str) -> Self {
        match command {
            "start" => RchatCommand::Start,
            "login" => RchatCommand::Login,
            "p2p" => RchatCommand::P2p,
            "chat" => RchatCommand::Chat,
            _ => {
                panic!("不支持的command:{}", command)
            }
        }
    }

    pub fn to_data_type(self) -> Vec<u8> {
        let v = self as u8;
        vec![v]
    }

    pub fn to_self(b: u8) -> Self {
        RchatCommand::index_enum(b as usize).unwrap()
    }
}
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TcpSideState {
    INIT,
    RUNNING,
    STOPPED,
}

pub struct TcpClientSide {
    local_addr: SocketAddr,
    server_side: Option<TcpServerSide>,
}

impl TcpClientSide {
    pub fn get_state(&self) -> &TcpSideState {
        self.server_side.as_ref().unwrap().get_state()
    }

    pub fn new(server_side_address: SocketAddr, factory: HandleProtocolFactory) -> Self {
        // 连接server端，得到stream
        let server_stream = TcpStream::connect(server_side_address).expect("连接server端失败!");

        // 从stream中得到本地使用的地址
        let local_addr = server_stream.local_addr().unwrap();

        info!("client使用端口地址:{}", local_addr.to_string());

        let mut client = TcpClientSide {
            local_addr,
            server_side: None,
        };

        // 用local_addr创建tcpServerSide
        let mut server = TcpServerSide::new(local_addr.to_string(), factory);

        let cache_data = ProtocolCacheData {
            stream: server_stream,
            data: None,
        };

        server.add_stream(server_side_address, cache_data);

        client.server_side = Some(server);

        client
    }

    // invoke this function , current thread will be loop to execute handle accept request.
    pub fn start(&mut self) {
        self.server_side.as_mut().unwrap().start();
    }
}

pub struct TcpServerSide {
    // addr必须是 "ip:port"的格式
    addr: String,
    factory: Arc<HandleProtocolFactory>,
    state: Arc<Mutex<TcpSideState>>,
    all_conn_cache: Arc<HashMap<SocketAddr, ProtocolCacheData>>,
}

impl TcpServerSide {
    /***
     ***    insert new value to all_conn_cache , but if already had key , then return false,else return ture.
     ***
     ***/
    pub fn add_stream(&mut self, addr: SocketAddr, stream_cache: ProtocolCacheData) -> bool {
        match self.all_conn_cache.contains_key(&addr) {
            // note: if already had key , can not insert new value
            true => {
                return false;
            }

            false => {
                self.all_conn_cache.insert(addr, stream_cache);
                return true;
            }
        }
    }

    pub fn new(addr: String, factory: HandleProtocolFactory) -> Self {
        TcpServerSide {
            addr,
            factory: Arc::new(factory),
            state: Arc::new(Mutex::new(TcpSideState::INIT)),
            all_conn_cache: Arc::new(Default::default()),
        }
    }

    pub fn get_state(&self) -> &TcpSideState {
        self.state.as_ref().into_inner().as_ref().unwrap()
    }

    pub fn start(&mut self) {
        self.start_server_accept();
        *self.state.lock().unwrap() = TcpSideState::RUNNING;
    }

    pub fn stop(&mut self) {
        *self.state.lock().unwrap() = TcpSideState::STOPPED;
    }

    fn start_server_accept(&mut self) {
        let state_lock = Arc::clone(&self.state);
        let addr = self.addr.clone();
        let arc_cach = Arc::clone(&self.all_conn_cache);
        let arc_factory = Arc::clone(&self.factory);

        let t = thread::spawn(move || {
            let listener = TcpListener::bind(addr).unwrap();
            println!("##########  TcpServer started! ###########");
            while state_lock.into_inner().unwrap() == TcpSideState::RUNNING {
                let (stream, address) = listener.accept().unwrap();
                parse_tcp_stream(stream, address, arc_cach, arc_factory);
            }
        });
        t.join();
        println!("##########  TcpServer stopped! ###########");
    }
}

pub struct ProtocolCacheData {
    stream: TcpStream,

    data: Option<Protocol>,
}

fn parse_tcp_stream(
    stream: TcpStream,
    address: SocketAddr,
    mut all_cache: Arc<HashMap<SocketAddr, ProtocolCacheData>>,
    mut factory: Arc<HandleProtocolFactory>,
) {
    let mut pca = match all_cache.remove(&address) {
        Some(mut t) => {
            match t.data {
                None => t.data = Some(Protocol::create_new()),
                Some(_) => {}
            }
            t
        }

        None => ProtocolCacheData {
            stream,
            data: Some(Protocol::create_new()),
        },
    };

    let mut buf = [0; 128];

    let mut remain = pca.stream.read(&mut buf).unwrap();

    let total_len = remain.clone();

    let mut index = 0;

    let buffer = buf.to_vec();

    while remain > 0 {
        let len = fill(
            pca.data.as_mut().unwrap(),
            &buffer,
            index.clone(),
            total_len.clone(),
        );

        remain -= len;

        index += len.clone();

        if pca.data.as_ref().unwrap().completion() {
            let resp = handle_pkg(
                pca.data.as_ref().unwrap(),
                address.clone(),
                Arc::clone(&factory),
            );

            if resp.is_some() {
                pca.stream
                    .write_all(&resp.unwrap())
                    .expect("stream send resp occurs fail !");
            }

            if remain > 0 {
                pca.data = Some(Protocol::create_new());
            }
        }
    }

    if !pca.data.as_ref().unwrap().completion() {
        all_cache.insert(address, pca);
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

fn handle_pkg(
    pkg: &Protocol,
    address: SocketAddr,
    mut factory: Arc<HandleProtocolFactory>,
) -> Option<Vec<u8>> {
    // convert bytes to struct by type
    let data_type = pkg.data_type.as_ref().unwrap()[0].clone();
    let command = RchatCommand::to_self(data_type);
    let handler = factory.get_handler(&command);
    handler.handle(address, pkg.data.as_ref().unwrap())
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
        time: 0,
    };

    bincode::serialize(&c).unwrap()
}
