pub enum r_error {
    // 内部错误
    InternalError(String),

    // handler注册失败
    HandlerRegistryFail(String),

    LoginFail(String),
}
