use std::collections::HashMap;

pub mod sandwich;

pub enum ProxyResponse {
    Sandwich(sandwich::StatusEndpointResponse)
}

impl ProxyResponse {
    pub fn to_support_data(&self) -> ProxySupportData {
        match self {
            ProxyResponse::Sandwich (data) => {
                let mut shards = HashMap::new();
                for manager in data.managers.iter() {
                    if manager.display_name != *"Anti Raid" {
                        continue; // Not for us
                    }

                    for v in manager.shard_groups.iter() {
                        for shard in v.shards.iter() {
                            let shard_id = shard[0];
                            let status = shard[1];
                            let latency = shard[2];
                            let guilds = shard[3];
                            let uptime = shard[4];
                            let total_uptime = shard[5];

                            shards.insert(shard_id, ShardConn {
                                status: match status {
                                    0 => "Idle".to_string(),
                                    1 => "Connecting".to_string(),
                                    2 => "Connected".to_string(),
                                    3 => "MarkedForClosure".to_string(),
                                    4 => "Closing".to_string(),
                                    5 => "Closed".to_string(),
                                    6 => "Erroring".to_string(),
                                    _ => "Unknown".to_string(),
                                },
                                real_latency: latency,
                                guilds,
                                uptime,
                                total_uptime,
                            });
                        }
                    }
                }
                
                ProxySupportData {
                    resp: ProxyResponse::Sandwich(data.clone()),
                    shard_conns: shards,
                }
            }
        }
    }
}

pub struct ShardConn {
    pub status: String,
    pub real_latency: i64,
    pub guilds: i64,
    pub uptime: i64,
    pub total_uptime: i64,
}

pub struct ProxySupportData {
    pub resp: ProxyResponse,
    pub shard_conns: HashMap<i64, ShardConn>,
}