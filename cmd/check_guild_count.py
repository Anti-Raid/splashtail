import sys
import os
import discord
from discord.ext.commands import AutoShardedBot as Bot

bot = Bot(command_prefix="!", intents=discord.Intents.default())

@bot.event
async def on_ready():
    guild_count = len(bot.guilds)
    print(f"Bot is in {guild_count} guilds")
    sys.exit(0)

bot.run(os.getenv("TOKEN"))