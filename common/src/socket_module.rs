use crate::biz_module::DefaultBizModule;
use crate::chat_protocol::{parse_tcp_stream, ProtocolCacheData};
use crate::config::TcpSocketConfig;
use log::info;
use std::collections::HashMap;
use std::fmt::Error;
use std::net::{SocketAddr, TcpListener};

static MAX_DATA_LEN: u64 = u32::MAX as u64;

pub trait SocketModule {
    // 发送字节数据到指定的地址
    fn send_bytes(&self, addr: SocketAddr, data: Vec<u8>) -> Result<(), Error>;

    // 指定地址的连接是否存在
    fn check_connected(&self, addr: SocketAddr) -> bool;

    // 查看当前SocketModule的状态
    fn check_socket_module_state(&self) -> SocketModuleState;

    // 启动,监听指定端口
    fn start(&mut self) -> Result<(), Error>;
}

pub struct DefaultSocketModule {
    share: DefaultBizModule,
    cache: HashMap<SocketAddr, ProtocolCacheData>,
    state: SocketModuleState,
}

impl DefaultSocketModule {
    pub fn init(share: DefaultBizModule) -> Self {
        DefaultSocketModule {
            share,
            cache: Default::default(),
            state: SocketModuleState::INIT,
        }
    }

    pub fn stop(&mut self) -> bool {
        self.state = SocketModuleState::STOPPED;
        return true;
    }
}

impl SocketModule for DefaultSocketModule {
    fn send_bytes(&self, addr: SocketAddr, data: Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    fn check_connected(&self, addr: SocketAddr) -> bool {
        todo!()
    }

    fn check_socket_module_state(&self) -> SocketModuleState {
        todo!()
    }

    fn start(&mut self) -> Result<(), Error> {
        // 读取配置文件，获取要监听的端口和地址
        let config = TcpSocketConfig::init_from_env();

        if self.state != SocketModuleState::INIT {
            panic!("DefaultSocketModule.state is not INIT, can not start!");
        }

        let listener = TcpListener::bind(config.get_url()).unwrap();

        self.state = SocketModuleState::RUNNING;

        info!("##########  DefaultSocketModule started! ###########");

        while self.state == SocketModuleState::RUNNING {
            let (stream, address) = listener.accept().unwrap();
            parse_tcp_stream(stream, address, &mut self.cache, &mut self.share);
        }

        println!("##########  DefaultSocketModule stopped! ###########");

        Ok(())
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SocketModuleState {
    INIT,
    RUNNING,
    STOPPING,
    STOPPED,
}
