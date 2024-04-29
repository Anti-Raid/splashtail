use std::collections::HashMap;

pub mod sandwich;


/// Faster version of botox member in guild that also takes into account the sandwich proxy layer
pub async fn member_in_guild(
    ctx: &botox::cache::CacheHttpImpl,
    reqwest_client: &reqwest::Client,
    guild_id: serenity::model::id::GuildId,
    user_id: serenity::model::id::UserId,
) -> Result<serenity::all::Member, crate::Error> {
    if crate::config::CONFIG.meta.sandwich_http_api.is_none() {
        let res = botox::cache::member_on_guild(ctx, guild_id, user_id, true).await?;
    
        let Some(res) = res else {
            return Err("Member not found".into());
        };

        return Ok(res);
    }

    let res = botox::cache::member_on_guild(ctx, guild_id, user_id, false).await?;

    if let Some(res) = res {
        return Ok(res);
    }

    // Part 2, try sandwich state
    let Some(ref proxy_url) = crate::config::CONFIG.meta.sandwich_http_api else {
        return Err("Sandwich proxy not configured".into());
    };

    let url = format!("{}/api/state?col=members&id={}&guild_id={}", proxy_url, user_id, guild_id);

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Resp {
        ok: bool,
        data: Option<serenity::all::Member>,
        error: Option<String>,
    }
    
    let resp = reqwest_client
    .get(&url)
    .send()
    .await?
    .json::<Resp>()
    .await?;

    if resp.ok {
        let Some(member) = resp.data else {
           return Err("Member not found".into());
        };

        return Ok(member);
    } else {
        log::warn!("Sandwich proxy returned error: {:?}", resp.error);
    }

    // Last resort, use botox to fetch from http and then update sandwich as well
    let res = botox::cache::member_on_guild(ctx, guild_id, user_id, true).await?;

    let Some(res) = res else {
        return Err("Member not found".into());
    };

    // Update sandwich with a POST
    let resp = reqwest_client
    .post(&url)
    .json(&res)
    .send()
    .await?;

    if !resp.status().is_success() {
        log::warn!("Failed to update sandwich proxy with member data: {:?}", resp.text().await);
    }

    Ok(res)
}

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