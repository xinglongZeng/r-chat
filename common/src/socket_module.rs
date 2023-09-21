use crate::biz_module::DefaultBizModule;
use crate::config::TcpSocketConfig;
use derive_more::Display;
use enum_index_derive::{EnumIndex, IndexEnum};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

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

        info!("##########  DefaultSocketModule started! ###########");

        while self.state == SocketModuleState::RUNNING {
            let (stream, address) = listener.accept().unwrap();
            parse_tcp_stream(stream, address, &mut self.cache, &mut self.share);
        }

        println!("##########  DefaultSocketModule stopped! ###########");

        Ok(())
    }
}

/** socket报文解析的module
 **/
trait ParseProtocolModule<T> {
    // 解析字节数据为协议报文数据
    fn parse_bytes_to_protocol(addr: SocketAddr, data: Vec<u8>) -> ParseResult;
}

//解析结果
struct ParseResult {
    // 是否解析完成
    finished: bool,
    // 已经解析出来的协议数据，注意，可能该协议数据并不完整
    protocol: Protocol,
    // 剩余的还未解析的字节数据
    remain: Vec<u8>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SocketModuleState {
    INIT,
    RUNNING,
    STOPPED,
}

/**
**  socket协议报文
**/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    // -----------------    head 区   ---------------

    // 版本号,1个字节
    pub version: Option<Vec<u8>>,

    //数据区的数据类型，1个字节
    pub data_type: Option<Vec<u8>>,

    // 数据区长度，4个字节
    pub data_len: Option<Vec<u8>>,

    // -------------------  数据区 ,最多有 2^ 32-1 个字节----------------
    pub data: Option<Vec<u8>>,
}

impl Protocol {
    pub fn to_vec(&mut self) -> Vec<u8> {
        let mut v = vec![];
        v.append(self.version.as_mut().unwrap());
        v.append(self.data_type.as_mut().unwrap());
        v.append(self.data_len.as_mut().unwrap());
        v.append(self.data.as_mut().unwrap());
        v
    }

    pub fn create_new() -> Self {
        Protocol {
            version: None,
            data_type: None,
            data_len: None,
            data: None,
        }
    }

    pub fn completion(&self) -> bool {
        for field_name in Self::get_all_filed_name() {
            // 由于tcp存在分包情况，所以有可能部分字段的数据不完整，所以进行判断时需要判断字节长度是否满足要求，长度为1个字节的的可以忽略
            if !self.check_field_fill(&field_name) {
                return false;
            }
        }

        true
    }

    // 获取所有字段名
    pub fn get_all_filed_name() -> [ProtocolFieldNameEnum; 4] {
        [
            ProtocolFieldNameEnum::version,
            ProtocolFieldNameEnum::data_type,
            ProtocolFieldNameEnum::data_len,
            ProtocolFieldNameEnum::data,
        ]
    }

    // 检查指定字段的数据是否填充完整
    pub fn check_field_fill(&self, field_key: &ProtocolFieldNameEnum) -> bool {
        let field = self.get_field(field_key);
        return field.is_some() && field.unwrap().len() == self.get_field_usize(field_key);
    }

    // 获取指定字段
    pub fn get_field(&self, field_name: &ProtocolFieldNameEnum) -> Option<&Vec<u8>> {
        let field = match field_name {
            ProtocolFieldNameEnum::version => self.version.as_ref(),
            ProtocolFieldNameEnum::data_type => self.data_type.as_ref(),
            ProtocolFieldNameEnum::data_len => self.data_len.as_ref(),
            ProtocolFieldNameEnum::data => self.data.as_ref(),
            _ => panic!("can not support field name : {field_name} "),
        };

        field
    }

    // 获取任意一个字段所需字节长度 . (data字段需要data_len字段先设置完成才行)
    pub fn get_field_usize(&self, field_key: &ProtocolFieldNameEnum) -> usize {
        match field_key {
            ProtocolFieldNameEnum::version | ProtocolFieldNameEnum::data_type => 1,

            ProtocolFieldNameEnum::data_len => 4,

            ProtocolFieldNameEnum::source_id | ProtocolFieldNameEnum::target_id => 8,

            ProtocolFieldNameEnum::data => self.calculate_data_len(),

            _ => panic!("can not support field name : {field_key} ."),
        }
    }

    //查询要填充满指定字段所需字节数
    pub fn get_diff_size(&self, field_name: &ProtocolFieldNameEnum) -> usize {
        let size = self.get_field_usize(field_name);
        let field = self.get_field(field_name);
        match field {
            None => size,
            Some(t) => size - t.len(),
        }
    }

    // 将bytes填充到指定字段。注意，这里进行填充时请自行确保数据长度的有效性
    pub fn fill_field(&mut self, field_name: &ProtocolFieldNameEnum, mut bytes: Vec<u8>) {
        let field = self.get_field_mut(field_name);

        match field {
            Some(t) => t.append(&mut bytes),
            None => {
                let v = Some(bytes);
                match field_name {
                    ProtocolFieldNameEnum::version => self.version = v,
                    ProtocolFieldNameEnum::data_type => self.data_type = v,
                    ProtocolFieldNameEnum::data_len => self.data_len = v,
                    ProtocolFieldNameEnum::data => self.data = v,
                    _ => {}
                }
            }
        }
    }

    // 获取指定字段 可变的
    pub fn get_field_mut(&mut self, field_name: &ProtocolFieldNameEnum) -> Option<&mut Vec<u8>> {
        let field = match field_name {
            ProtocolFieldNameEnum::version => self.version.as_mut(),
            ProtocolFieldNameEnum::data_type => self.data_type.as_mut(),
            ProtocolFieldNameEnum::data_len => self.data_len.as_mut(),
            ProtocolFieldNameEnum::data => self.data.as_mut(),
            _ => panic!("can not support field name : {field_name} ."),
        };

        field
    }

    // 根据data_len字段计算出data区有多少个字节
    fn calculate_data_len(&self) -> usize {
        if !self.check_field_fill(&ProtocolFieldNameEnum::data_len) {
            panic!("field <data_len> not be set value !");
        } else {
            let x = self.data_len.as_ref().unwrap();
            let value = [x[0].clone(), x[1].clone(), x[2].clone(), x[3].clone()];
            transform_array_of_u8_to_u32(value) as usize
        }
    }
}

fn transform_array_of_u8_to_u32(x: [u8; 4]) -> u32 {
    u32::from_be_bytes(x)
}

// 根据data大小计算出Protocol的data_len字段的字节表示
pub fn calculate_len_by_data(data: &Vec<u8>) -> Vec<u8> {
    let len = data.len() as u64;
    if len > MAX_DATA_LEN {
        panic!("data.len() over MAX_DATA_LEN !");
    }

    (len as u32).to_be_bytes().to_vec()
}

#[derive(Debug, Clone, Display, EnumIndex, IndexEnum)]
pub enum ProtocolFieldNameEnum {
    version,
    data_type,
    data_len,
    source_id,
    target_id,
    data,
}

pub struct ProtocolCacheData {
    stream: TcpStream,

    data: Option<Protocol>,
}

fn parse_tcp_stream(
    stream: TcpStream,
    address: SocketAddr,
    all_cache: &mut HashMap<SocketAddr, ProtocolCacheData>,
    default_biz: &mut DefaultBizModule,
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
            let resp = default_biz.handle_pkg(pca.data.as_ref().unwrap());
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
