use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct CmdArgs {
    #[clap(long, short)]
    pub shards: Option<Vec<u16>>,
    #[clap(long, short)]
    pub shard_count: Option<u16>,
}
