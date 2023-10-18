use derive_more::Display;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Display)]
pub struct CliOpt {
    // support command :    start, login,chat,p2p
    #[structopt(short, long)]
    pub command: String,
}
