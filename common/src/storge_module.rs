use crate::{Module, ModuleEngine};
use std::fmt::Error;
use std::sync::Arc;

// todo: 存取数据的格式和逻辑
trait StorageModule<T: Module + Sized + 'static>: Module {
    // 存数据 todo: 入参
    fn storage_data(share: Arc<ModuleEngine<T>>) -> Result<(), Error>;

    // 取数据,但存储的数据还在. todo:入参、返回值
    fn get_data(share: Arc<ModuleEngine<T>>) -> Result<(), Error>;
}
