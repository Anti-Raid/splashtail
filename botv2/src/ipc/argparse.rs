/// This crate parses down the mewld arguments down to a simple MewldCmdArgs struct
use once_cell::sync::Lazy;

pub static MEWLD_ARGS: Lazy<MewldCmdArgs> = Lazy::new(|| {
    let args: Vec<String> = std::env::args().collect();
    MewldCmdArgs::parse_argv(&args).expect("Failed to parse mewld arguments")
});

/*
        l.Dir+"/"+l.Config.Module,
        mutils.ToPyListUInt64(i.Shards),
        mutils.UInt64ToString(l.ShardCount),
        strconv.Itoa(i.ClusterID),
        cm.Name,
        l.Dir,
        strconv.Itoa(len(l.Map)),
        state.Config.Sites.API.Parse(),
        l.Config.RedisChannel,
        config.CurrentEnv,
     */
#[derive(Debug, Clone)]
pub struct MewldCmdArgs {
    pub shards: Vec<u16>,
    pub shard_count: u16,
    pub cluster_id: u16,
    pub cluster_name: String,
    pub base_dir: String,
    pub cluster_count: u16,
    pub splashtail_url: String,
    pub mewld_redis_channel: String,
    pub current_env: String,
}

impl MewldCmdArgs {
    pub fn parse_argv(args: &[String]) -> Result<Self, crate::Error> {
        if args.len() != 10 {
            return Err(r#"Invalid number of arguments
            
Expected arguments: [program name] <shards> <shard_count> <cluster_id> <cluster_name> <base_dir> <cluster_count> <splashtail_url> <mewld_redis_channel> <env>
            "#.into());
        }

        let shards: Vec<u16> = serde_json::from_str(&args[1])?;
        let shard_count: u16 = str::parse(&args[2])?;
        let cluster_id: u16 = str::parse(&args[3])?;
        let cluster_name: String = args[4].clone();
        let base_dir: String = args[5].clone();
        let cluster_count: u16 = str::parse(&args[6])?;
        let splashtail_url: String = args[7].clone();
        let mewld_redis_channel: String = args[8].clone();
        let current_env: String = args[9].clone();

        Ok(Self {
            shards,
            shard_count,
            cluster_id,
            cluster_name,
            base_dir,
            cluster_count,
            splashtail_url,
            mewld_redis_channel,
            current_env,
        })
    }
}