use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{env, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginReqData {
    pub account: String,
    pub pwd: String,
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
