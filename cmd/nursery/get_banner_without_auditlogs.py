# An attempt to get the performer of an action without relying on audit logs
#
# Use case: Larger nukes cause discords audit logging system to fail
import discord
from discord.ext.commands import AutoShardedBot as Bot

import discord
import discord.gateway
import discord.http
import subprocess
import logging
import yarl
import time
import pathlib
from ruamel.yaml import YAML

# Get git repo root
def get_git_repo_root():
    return subprocess.check_output(["git", "rev-parse", "--show-toplevel"]).decode("utf-8").strip()

git_repo_root = get_git_repo_root()

with open(f"{git_repo_root}/config.yaml") as f:
    yaml_dict = YAML().load(f)

proxy_url = yaml_dict["meta"]["proxy"]["staging"]
token = yaml_dict["discord_auth"]["token"]

def patch_with_gateway(env_gateway):
    class ProductionHTTPClient(discord.http.HTTPClient):
        async def get_gateway(self, **_):
            return f"{env_gateway}?encoding=json"

        async def get_bot_gateway(self, **_):
            try:
                data = await self.request(discord.http.Route("GET", "/gateway/bot"))
            except discord.HTTPException as exc:
                raise discord.GatewayNotFound() from exc
            return data["shards"], f"{env_gateway}?encoding=json&v=9"

    class ProductionDiscordWebSocket(discord.gateway.DiscordWebSocket):
        DEFAULT_GATEWAY = yarl.URL(env_gateway)

        def is_ratelimited(self):
            return False

    class ProductionBot(Bot):
        async def before_identify_hook(self, shard_id, *, initial):
            pass

        def is_ws_ratelimited(self):
            return False

    class ProductionReconnectWebSocket(Exception):
        def __init__(self, shard_id, *, resume=False):
            self.shard_id = shard_id
            self.resume = False
            self.op = "IDENTIFY"

    discord.http.HTTPClient.get_gateway = ProductionHTTPClient.get_gateway
    discord.http.HTTPClient.get_bot_gateway = ProductionHTTPClient.get_bot_gateway
    discord.gateway.DiscordWebSocket.DEFAULT_GATEWAY = ProductionDiscordWebSocket.DEFAULT_GATEWAY
    discord.gateway.DiscordWebSocket.is_ratelimited = ProductionDiscordWebSocket.is_ratelimited
    discord.gateway.ReconnectWebSocket.__init__ = ProductionReconnectWebSocket.__init__
    return ProductionBot

bot = patch_with_gateway("ws://127.0.0.1:3600")

client = bot(command_prefix="!", intents=discord.Intents.all(), chunk_guilds_at_startup=True)

@client.event
async def on_ready():
    print(f"We have logged in as {client.user}")

@client.event
async def on_member_ban(guild: discord.Guild, member: discord.Member):
    if isinstance(member, discord.User):
        return
    
    # First, find all users in the guild who *can* ban members and whose highest role is greater than the member
    possible_members = []
    if len(guild.members) < (guild.approximate_member_count or guild.member_count):
        async for mem in guild.fetch_members():
            if mem.guild_permissions.ban_members and mem.top_role > member.top_role:
                possible_members.append([mem.id, mem.name])
    else:
        for mem in guild.members:
            if mem.guild_permissions.ban_members and mem.top_role > member.top_role:
                possible_members.append([mem.id, mem.name])
    
    print(f"Ban possibilities: {possible_members} [banned member: {member}]")

    # Save to ban_possibilities/{gid}-{member}-{ts}.json
    pathlib.Path(f"{git_repo_root}/cmd/nursery/ban_possibilities").mkdir(parents=True, exist_ok=True)
    with open(f"{git_repo_root}/cmd/nursery/ban_possibilities/{guild.id}-{member.id}-{time.time()}.json", "w") as f:
        YAML().dump({"possible": possible_members, "member": [member.id, member.name]}, f)

@client.command()
async def ping(ctx):
    await ctx.send(f"Pong! {ctx.bot.latency} {len(ctx.bot.guilds)}")
    
client.run(token, log_level=logging.INFO)