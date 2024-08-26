import discord
import logging

logging.getLogger("discord.gateway").setLevel(logging.DEBUG)

print(discord.gateway)

bot = discord.AutoShardedClient(intents=discord.Intents.default())

@bot.event
async def on_ready():
    print(bot.user.name)

bot.run("ODU4MzA4OTY5OTk4OTc0OTg3.GTK8za.pqsmroCuWuarOtaQIlYJczbBgn8B4QrRiBJDYM")
