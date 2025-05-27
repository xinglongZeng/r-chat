use crate::base::{RchatCommand, RchatCommandRequest, RcommandResult};
use derive_more::Display;
use std::io::{Stdout, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use structopt::StructOpt;
#[derive(StructOpt, Debug, Display)]
// #[structopt(name = "basic")]
pub struct CliOpt {
    // support command :    start, login,chat,p2p
    #[structopt(short, long)]
    pub command: String,
}

// 开启交互式的cli
pub fn start_interactive_cli(
    command_tx: Sender<RchatCommandRequest>,
    command_result_rx: Receiver<RcommandResult>,
) {
    loop {
        // use the `>` character as the prompt
        // need to explicitly flush this to ensure it prints before read_line
        print!("> ");

        let mut out = std::io::stdout();
        
        let _ = out.flush();
        
        let mut input = String::new();
        
        std::io::stdin().read_line(&mut input).unwrap();
        
        let mut parts = input.trim().split_whitespace();

        let command_option = parts.next();
        
        if command_option == None {
            let _ = out.write_all("command is null ! \n".as_bytes());
            let _ = out.flush();
            continue
        }
        

        let command = RchatCommand::from_string(command_option.unwrap().trim());
        
        let args_option = parts.next();

        if args_option == None {
            let _ = out.write_all("args is null ! \n".as_bytes());
            let _ = out.flush();
            continue
        }
        
        let args_string = args_option.unwrap().to_string();
        
        let req = RchatCommandRequest{
            command,
            args: args_string,
        };
        

        let _ = out.write_all(format! {"输入的参数:{:?}\n", req}.as_bytes());

        // 通过消息通道发送到业务线程处理
        let send_result = command_tx.send(req);

        if send_result.is_err() {
            let _ = out.write_all(format! {"无法执行-[command_tx.send  fail]! cause:{:?}\n", send_result.err().unwrap()}.as_bytes());
        } else {
            let _ = out.write_all("执行中，请稍等...\n".as_bytes());
        }

        // 等待执行结果
        let execute_result = command_result_rx.recv().unwrap();

        match execute_result.is_success {
            true => {
                let _ = out.write_all("执行完成\n".as_bytes());
            }
            false => {
                let _ = out
                    .write_all(format! {"执行失败! cause:{}\n", execute_result.err_msg}.as_bytes());
            }
        }
        let _ = out.flush();
    }
}

/// 开启子线程来监听cli的参数，然后通过消息通道的方式讲参数传送到处理socket的线程
pub fn start_cli_listen(
    command_tx: Sender<RchatCommandRequest>,
    command_result_rx: Receiver<RcommandResult>,
) -> JoinHandle<()> {
    let cli_task = thread::spawn(move || {
        start_interactive_cli(command_tx, command_result_rx);
    });
    cli_task
}
