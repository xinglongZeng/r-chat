use common::config::TcpSocketConfig;
use service::userinfo_service::Service;
use std::sync::Arc;

pub use service::sea_orm;
pub use service::userinfo_dao;
pub use service::userinfo_service;

pub fn start_webserver_userinfo(
    user_service: Arc<Service>,
    socket_config: TcpSocketConfig,
) -> std::io::Result<()> {
    api::api_start_web_server_new(user_service, socket_config)
}
