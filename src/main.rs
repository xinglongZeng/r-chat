use r_chat::net::{create_factory, start_tcp_server};

#[tokio::main]
async fn main() {
    let factory = create_factory();

    let addr="localhost:9999";

    start_tcp_server(addr,&factory)
        .await
        .unwrap();

}
