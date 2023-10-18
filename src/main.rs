use common::cli::CliOpt;
use common::structopt::StructOpt;
pub fn main() {
    let cli = CliOpt::from_args();
    println!("cli:{:?}", cli);

    // server::start_server();
}

#[cfg(test)]
mod tests {
    use common::cli::CliOpt;
    use common::structopt::StructOpt;

    #[test]
    fn start_client_side() {
        // 调用start_server函数时会阻塞当前线程来一直处理server端的accept的处理，所以最好另开线程来调用start_server
        // server::start_server();

        // start_client也会阻塞当前线程，原因同上面一样
        client::start_client_mode();
    }

    #[test]
    fn test_cliopt() {
        let cli = CliOpt::from_args();
    }
}
