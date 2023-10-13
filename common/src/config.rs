use std::env;

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

pub struct WebSocketConfig {
    pub web_host: String,
    pub web_port: String,
}

impl WebSocketConfig {
    pub fn init_from_env() -> Self {
        dotenvy::dotenv().ok();

        let web_host = env::var("WEB_HOST").expect("WEB_HOST is not set in .env file");

        let web_port = env::var("WEB_PORT").expect("WEB_PORT is not set in .env file");

        WebSocketConfig { web_host, web_port }
    }

    pub fn get_url(&self) -> String {
        format!("{}:{}", self.web_host, self.web_port)
    }
}

pub struct ClientDefaultConfig {
    pub account_save_path: String,
}

impl ClientDefaultConfig {
    pub fn init_from_env() -> Self {
        dotenvy::dotenv().ok();
        let account_save_path = env::var("CLIENT_ACCOUNT_SAVE_PATH")
            .expect("CLIENT_ACCOUNT_SAVE_PATH is not set in .env file");
        ClientDefaultConfig { account_save_path }
    }
}
