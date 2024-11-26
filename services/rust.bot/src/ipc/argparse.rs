use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct CmdArgs {
    #[clap(long)]
    pub shards: Option<Vec<u16>>,
    #[clap(long)]
    pub shard_count: Option<u16>,
}
