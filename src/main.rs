pub fn main() {
    server::start_server();
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};

    #[test]
    fn start_client_side() {
        // 调用start_server函数时会阻塞当前线程来一直处理server端的accept的处理，所以最好另开线程来调用start_server
        // server::start_server();

        // start_client也会阻塞当前线程，原因同上面一样
        client::start_client();
    }
}
