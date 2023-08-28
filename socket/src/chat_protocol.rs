use core::cmp::Eq;
use derive_more::Display;
use enum_index::{EnumIndex, IndexEnum};
use enum_index_derive::{EnumIndex, IndexEnum};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

static MAX_DATA_LEN: u64 = u32::MAX as u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct P2pData {
    pub biz: P2pDataType,
    pub body: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum P2pDataType {
    GetIpV4Req,
    GetIpV4Resp,
    TrtConnectReq,
    TrtConnectResp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIpV4Req {
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetIpV4Resp {
    pub account: String,
    // 符合 ip:port格式
    pub ipv4: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatData {
    pub from_account: String,
    pub to_account: String,
    pub contents: Vec<ChatContent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChatContent {
    Text(ChatTextContent),
    File(ChatFileContent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatTextContent {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatFileContent {
    pub file_name: String,
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
}

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

#[derive(Debug, Clone, Display, EnumIndex, IndexEnum)]
pub enum ProtocolFieldNameEnum {
    version,
    data_type,
    data_len,
    source_id,
    target_id,
    data,
}

#[derive(Debug, Clone, EnumIndex, IndexEnum, Hash, Serialize, Deserialize)]
pub enum ChatCommand {
    Login_req,
    Login_resp,
    Chat,
    P2p,
}

impl PartialEq<Self> for ChatCommand {
    fn eq(&self, other: &Self) -> bool {
        self.enum_index() == other.enum_index()
    }
}

impl Eq for ChatCommand {}

impl ChatCommand {
    pub fn to_data_type(self) -> Vec<u8> {
        let v = self as u8;
        vec![v]
    }

    pub fn to_self(b: u8) -> Self {
        ChatCommand::index_enum(b as usize).unwrap()
    }
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
