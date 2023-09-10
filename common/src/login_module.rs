use crate::biz_module::{LoginDataEnum, LoginRespData};
use crate::{Module, ModuleEngine, ModuleNameEnum};
use std::any::Any;
use std::fmt::Error;
use std::fs;
use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::rc::Weak;

pub trait LoginModule: Module {
    fn handle_login_req(&self, req: LoginDataEnum) {
        panic!("暂未实现该函数 [handle_login_req]!");
    }

    // 处理登录响应
    fn handle_login_resp(&self, resp: LoginDataEnum) {
        panic!("暂未实现该函数 [handle_login_resp]!");
    }

    fn get_login_info(&self) -> Option<&LoginRespData>;

    fn check_token_timeout(&self) -> Result<bool, Error>;
}

struct DefaultClientLoginModule<T: Any + Module + Sized + 'static> {
    share: Weak<ModuleEngine<T>>,
    // 账户信息存储路径
    save_path: String,
    // 缓存的账户信息
    cache_account_info: Option<LoginRespData>,
}

impl<T: Any + Module + Sized + 'static> Module for DefaultClientLoginModule<T> {
    fn get_module_name() -> ModuleNameEnum {
        ModuleNameEnum::Biz
    }
}

impl<T: Any + Module + Sized + 'static> LoginModule for DefaultClientLoginModule<T> {
    fn handle_login_resp(&mut self, resp: LoginDataEnum) {
        match resp {
            LoginDataEnum::RespData(t) => {
                // 存储账户信息到文件
                save_account_info(&self.save_path, &t);
                //  存储账户信息到缓存
                self.cache_account_info = Some(t);
            }

            _ => {
                panic!("resp not is LoginDataEnum::RespData !");
            }
        }
    }

    fn get_login_info(&self) -> Option<&LoginRespData> {
        match &self.cache_account_info {
            Some(t) => Some(&t),

            None => None,
        }
    }

    fn check_token_timeout(&self) -> Result<bool, Error> {
        match &self.cache_account_info {
            None => Err(panic!("账户信息为空!")),
            Some(_) => {
                // todo: 检查token是否超时，目前token的生成规则还未定,暂时返回false
                Ok(false)
            }
        }
    }
}

fn save_account_info(path: &String, data: &LoginRespData) {
    // 1. 转换data为字节
    let mut byte_result = bincode::serialize(data).unwrap().as_slice();

    // 2. 将字节数据存储到文件中
    fs::create_dir_all(path).expect("创建account存储目录失败!");
    let file_name = format!("{}/{}", path, data.account);
    let file = File::create(file_name)?;
    file.write_all_at(byte_result, 0)?;
}
