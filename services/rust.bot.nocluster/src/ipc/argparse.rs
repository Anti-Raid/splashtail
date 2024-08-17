use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct CmdArgs {
    #[clap(long, short)]
    pub shards: Option<Vec<u16>>,
    #[clap(long, short)]
    pub shard_count: Option<u16>,
    #[clap(long, short, default_value_t = 0)]
    pub cluster_id: u16,
    #[clap(long, short, default_value = "Cluster 0")]
    pub cluster_name: String,
    #[clap(long, short, default_value_t = 1)]
    pub cluster_count: u16,
    #[clap(long, short, default_value = "staging")]
    pub current_env: String,
    #[clap(long, short, default_value = "animus_magic-staging")]
    pub animus_magic_channel: String,
}
