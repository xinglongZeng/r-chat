use std::fmt::Error;

// todo: 存取数据的格式和逻辑
pub trait StorageModule {
    // 存数据 todo: 入参
    fn storage_data(&self) -> Result<(), Error>;

    // 取数据,但存储的数据还在. todo:入参、返回值
    fn get_data(&self) -> Result<(), Error>;
}
