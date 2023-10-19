use common::base::{handle_rx, RchatCommand, RcommandResult, TcpClientSide};
use common::cli::CliOpt;
use common::login_module::{BizResult, ClientLoginModule, DefaultLoginHandler, LoginRespData};
use common::protocol_factory::HandleProtocolFactory;
use common::structopt::StructOpt;
use env_logger::Env;
use log::warn;
use std::fmt::Error;
use std::fs::File;
use std::io::{self, Write};
use std::net::{SocketAddr, SocketAddrV4};
use std::os::unix::fs::FileExt;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::{env, fs, thread};

pub fn start_client_mode() {
    // get env vars   读取.env文件中的变量，相当于读取配置文件
    dotenvy::dotenv().ok();

    // set logger level to debug
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // start trace info collect.  开启堆栈信息收集
    // tracing_subscriber::fmt::init();

    // 通道1，接受和处理command
    let (command_tx, command_rx) = mpsc::channel();

    // 通道2，接受和处理command的执行结果
    let (command_result_tx, command_result_rx) = mpsc::channel();

    // start_client_socket();

    let mut client = create_client_side();

    let task1 = thread::spawn(move || {
        client.start();
    });

    let task2 = handle_rx(command_rx, command_result_tx);

    let task3 = start_cli_listen(command_tx, command_result_rx);

    task1.join().expect("task join for client.start fail ! ");
    task2.join().expect("task join for handle_rx fail ! ");
    task3
        .join()
        .expect("task join for start_cli_listen fail ! ");
}

/// 开启子线程来监听cli的参数，然后通过消息通道的方式讲参数传送到处理socket的线程
fn start_cli_listen(
    command_tx: Sender<RchatCommand>,
    command_result_rx: Receiver<RcommandResult>,
) -> JoinHandle<()> {
    let cli_task = thread::spawn(move || {
        loop {
            let cli = CliOpt::from_args();

            println!("接收到的command参数:{:?}", cli);

            let command = RchatCommand::from_string(cli.command.as_str());

            // 通过消息通道发送到主线程处理
            let send_result = command_tx.send(command.clone());
            println!("command 发送完成. {:?}", command);

            if send_result.is_err() {
                println!("command channel send fail! command:{:?}", command);
            }
            // 从通道接收执行结果
            let handle_result = command_result_rx.recv().unwrap();
            // 处理 result
            handle_command_result(handle_result);
        }
    });

    cli_task
}

// 处理command的执行结果
fn handle_command_result(result: RcommandResult) {
    println!("开始处理 RcommandResult: {:?}", result);
    // 获得 stdout 实体
    let stdout = io::stdout();

    // 可选: 把  stdout 的 控制权 包裹进一个 buffer
    let mut handle = io::BufWriter::new(stdout);

    // 终端上输出执行结果
    writeln!(handle, "command-result: {:?}", result).expect("输出commande结果到终端失败!");
}

// 创建TcpClientSide
pub fn create_client_side() -> TcpClientSide {
    let factory = create_factory();

    let server_addr = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS is not set in .env file");

    let server_socket = SocketAddrV4::from_str(server_addr.as_str()).unwrap();

    let client = TcpClientSide::new(SocketAddr::V4(server_socket), factory);

    client
}

fn start_client_socket() {
    let mut client = create_client_side();
    client.start();
}

fn create_factory() -> HandleProtocolFactory {
    // login handler
    let login_handler = create_default_client_login_handler();
    // todo: chat handler
    // todo: p2p handler

    let mut factory = HandleProtocolFactory::new();
    factory.registry_handler(RchatCommand::Login, login_handler);
    factory
}

fn create_default_client_login_handler() -> Box<DefaultLoginHandler> {
    let client_login = DefaultClientLoginModule::init_from_env();
    Box::new(DefaultLoginHandler::new(
        false,
        None,
        Some(Box::new(client_login)),
    ))
}

pub struct DefaultClientLoginModule {
    // 账户信息存储路径
    save_path: String,
    // 缓存的账户信息
    cache_account_info: Option<LoginRespData>,
}

impl DefaultClientLoginModule {
    pub fn init_from_env() -> Self {
        dotenvy::dotenv().ok();
        let save_path = env::var("CLIENT_ACCOUNT_SAVE_PATH")
            .expect("CLIENT_ACCOUNT_SAVE_PATH is not set in .env file");
        DefaultClientLoginModule {
            save_path,
            cache_account_info: None,
        }
    }

    fn handle_login_resp(&mut self, resp: LoginRespData) {
        // 存储账户信息到文件
        let cache_data = save_account_info(&self.save_path, resp);
        //  存储账户信息到缓存
        self.cache_account_info = Some(cache_data);
    }

    fn get_login_cache_info(&self) -> Option<LoginRespData> {
        match &self.cache_account_info {
            None => None,
            Some(t) => Some(t.clone()),
        }
    }
    fn check_token_timeout(&self) -> Result<bool, Error> {
        match &self.cache_account_info {
            None => {
                panic!("账户信息为空!");
            }
            Some(_) => {
                // todo: 检查token是否超时，目前token的生成规则还未定,暂时返回false
                Ok(false)
            }
        }
    }
}

impl ClientLoginModule for DefaultClientLoginModule {
    fn handle_login_biz_resp(&mut self, resp: BizResult<LoginRespData>) {
        //  如果登录成功则存储获取到的账户信息，否则日志打印错误信息
        if resp.is_success {
            self.handle_login_resp(resp.data.unwrap());
        } else {
            warn!("登录失败,原因:{}", resp.msg.unwrap());
        }
    }
}

fn save_account_info(path: &String, data: LoginRespData) -> LoginRespData {
    // 1. 转换data为字节
    let byte_result = bincode::serialize(&data);

    // 2. 将字节数据存储到文件中
    fs::create_dir_all(path).expect("创建account存储目录失败!");
    let file_name = format!("{}/{}", path, &data.account);
    let file = File::create(file_name).unwrap();
    file.write_all_at(byte_result.unwrap().as_slice(), 0)
        .unwrap();
    data
}
