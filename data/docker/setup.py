#!/usr/bin/env python3
"""Sets up Docker Config"""
import os
import secrets
import base64
import json
import pathlib

os.mkdir("./data/dockerconf")

# Get bot token, client id and client secret
def get_var(prompt: str, env_var: str) -> str:
    """Gets a variable from the user or environment"""
    value = os.getenv(env_var)
    if not value:
        while True:
            value = input(f"{prompt}: ")

            if value:
                break
            print(f"{prompt} cannot be empty")

    if not value:
        print(f"{prompt} cannot be empty")
        exit(1)

    return value.replace(" ", "").replace("\n", "")

realtoken = get_var("Bot Token", "BOT_TOKEN")
client_id = get_var("Client ID", "CLIENT_ID")
client_secret = get_var("Client Secret", "CLIENT_SECRET")
sandwich_alert_webhook = get_var("Alert Webhook (for sandwich)", "SANDWICH_ALERT_WEBHOOK")

# Create some needed secrets
faketoken = f"{base64.b64encode(client_id.encode()).decode()}.{secrets.token_urlsafe(32)}.{secrets.token_urlsafe(32)}"
s3_access_key = secrets.token_urlsafe(32)
s3_secret_key = secrets.token_urlsafe(64)
vapid_public_key = secrets.token_urlsafe(32)
vapid_private_key = secrets.token_urlsafe(64)
dp_secret = secrets.token_urlsafe(128)

BASE_CONFIG_FILE = f"""
discord_auth:
  token: "{faketoken}" # Discord Bot Token
  client_id: "{client_id}" # Discord Client ID - Main
  client_secret: "{client_secret}" # Discord Client Secret - Main
  allowed_redirects: # Change afterwards if desired
    - http://localhost:5173/authorize
    - http://localhost:5174/authorize
    - https://v6-beta.antiraid.xyz/authorize
    - https://antiraid.xyz/authorize
  public_bot: 
    staging: true # Still in beta 
    prod: true # Public Bot
  dp_secret: {dp_secret}
  root_users:
    - "728871946456137770" # burgerking
    - "564164277251080208" # select
    - "775855009421066262" # ilief

sites:
  frontend: https://antiraid.xyz # Staging value
  api: http://localhost:5600 # Staging value
  docs: https://docs.antiraid.xyz # Documentation URL

channels:
  apps: "1216401068276060160" # Apps Channel, should be a staff only channel
  ban_appeals: "1222823352771678249" # Ban Appeals Channel

roles:
  apps: "1222825827071426571" # Apps pings
  awaiting_staff: "1222827385745838131"

japi:
  key: SOMETHING_FIX_LATER # JAPI Key. Get it from https://japi.rest

notifications:
  vapid_public_key: "{vapid_public_key}" # Vapid Public Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)
  vapid_private_key: "{vapid_private_key}" # Vapid Private Key (https://www.stephane-quantin.com/en/tools/generators/vapid-keys)

servers:
  main: "1064135068928454766" # Main Server ID

meta:
  web_disable_ratelimits: true
  postgres_url: postgres://antiraid:AnTiRaId123!@postgres:5432/antiraid # Postgres URL
  redis_url: redis://api_redis:6379/1 # Staging value
  jobserver_port: 5602
  port: 5600 
  bot_port: 10000
  cdn_path: /failuremgmt/cdn/antiraid # CDN Path
  secure_storage: /failuremgmt/sec/antiraid # Blob Storage URL
  urgent_mentions: <@&1061643797315993701> # Urgent mentions
  proxy: http://nirn_proxy:3221 # Proxy URL
  support_server_invite: https://discord.gg/9BJWSrEBBJ
  sandwich_http_api: http://sandwich:29334
  default_error_channel: "1234567890" # Change this

object_storage:
  type: s3-like # Type of object storage. Can be s3-like or local
  base_path: 
  endpoint: seaweed:8333 # Endpoint for Seaweed
  cdn_endpoint: $DOCKER:localhost:5601 # CDN Endpoint for Seaweed
  access_key: {s3_access_key} # Access Key for Seaweed
  secret_key: {s3_secret_key} # Secret Key for Seaweed
  secure: false
  cdn_secure: false

base_ports:
  jobserver_base_addr: http://jobserver
  bot_base_addr: http://bot
  jobserver_bind_addr: 0.0.0.0
  bot_bind_addr: 0.0.0.0
  jobserver: 30000
  bot: 20000
  template_worker_base_addr: 0.0.0.0
  template_worker_addr: template-worker
  template_worker_port: 60000
  template_worker_bind_addr: 0.0.0.0
"""

print("Saving config.docker.yaml")
with open("config.docker.yaml", "w") as f:
    f.write(BASE_CONFIG_FILE)

# Save secrets.docker.json with <faketoken>:<token>
nirn_secrets = {
    faketoken: realtoken,
}

print("Saving secrets.docker.json")
with open("./data/dockerconf/secrets.docker.json", "w") as f:
    json.dump(nirn_secrets, f, indent=4)

SANDWICH_YAML = f"""
identify:
    url: ""
    headers: {{}}
producer:
    type: websocket
    configuration:
        address: 0.0.0.0:3600
        expectedtoken: "{faketoken}"
http:
    oauth:
        clientid: "{client_id}"
        clientsecret: "{client_secret}"
        endpoint:
            authurl: https://discord.com/api/oauth2/authorize?prompt=none
            deviceauthurl: ""
            tokenurl: https://discord.com/api/oauth2/token
            authstyle: 0
        redirecturl: http://splashtail-sandwich.antiraid.xyz/callback
        scopes:
            - identify
            - email
    user_access: # Change this later
        - "USER1"
        - "USER2"
        - "USER3"
webhooks:
    - {sandwich_alert_webhook}
managers:
    - identifier: antiraid
      virtual_shards:
        enabled: true
        count: 30
        dm_shard: 0
      producer_identifier: antiraid_producer
      friendly_name: Anti Raid
      token: {realtoken}
      auto_start: true
      disable_trace: true
      bot:
        default_presence:
            status: online
            activities:
                - timestamps: null
                  applicationid: null
                  party: null
                  assets: null
                  secrets: null
                  flags: null
                  name: Listening to development of Anti-Raid v6 
                  url: null
                  details: null
                  state: Listening to development of Anti-Raid v6
                  type: 1
                  instance: null
                  createdat: null
            since: 0
            afk: false
        intents: 20031103
        chunk_guilds_on_startup: false
      caching:
        cache_users: true
        cache_members: true
        store_mutuals: true
      events:
        event_blacklist: []
        produce_blacklist: []
      messaging:
        client_name: antiraid
        channel_name: sandwich
        use_random_suffix: true
      sharding:
        auto_sharded: true
        shard_count: 0
        shard_ids: ""
"""

# Save to sandwich.docker.yaml
print("Saving sandwich.docker.yaml")
with open("./data/dockerconf/sandwich.docker.yaml", "w") as f:
    f.write(SANDWICH_YAML)

# Finally seaweed
data_path = pathlib.Path("data") / "docker" / "seaweed"
data_path.mkdir(parents=True, exist_ok=True)

s3_json = {
  "identities":  [
    {
      "name":  "antiraid",
      "credentials":  [
        {
          "accessKey":  s3_access_key,
          "secretKey":  s3_secret_key
        }
      ],
      "actions":  [
        "Read",
        "Write",
        "List",
        "Tagging",
        "Admin"
      ],
    },
  ],
  "accounts":  []
}

print("Saving data/docker/seaweed/s3.json")
with open(data_path / "s3.json", "w") as f:
    json.dump(s3_json, f, indent=4)

print("Done! Now run `docker compose up` to start the containers.")
