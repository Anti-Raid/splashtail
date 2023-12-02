import { EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { BotStaffPerms, Command, FinalResponse } from "../core/client";
import { postStats } from "../core/common/poststats";
import { getServerCount, getShardCount, getUserCount } from "../core/common/counts"

let command: Command = {
    userPerms: [],
    botPerms: [],
    botStaffPerms: [BotStaffPerms.Owner],
    interactionData: new SlashCommandBuilder()
    .setName("poststats")
	.setDescription("Post stats to all of our lists"),
    execute: async (ctx) => {
        await ctx.reply({
            embeds: [
                new EmbedBuilder()
                    .setColor("Orange")
                    .setDescription(
                        `Posting stats to all bot lists...`
                    ),
            ],
        })

        let variables = {
            servers: await getServerCount(ctx.client),
            shards: await getShardCount(ctx.client),
            members: await getUserCount(ctx.client),
            botId: ctx.client.user.id,
        }    

        ctx.client.logger.info("PostStats", variables)

        let results: { [key: string]: Response } = {}

        for (const botList of ctx.client.config.bot_lists) {
            if(!botList?.post_stats?.enabled) continue;

            let res = await postStats(variables, ctx.client, botList, botList.post_stats)

            results[botList.name] = res
        }

        let embed = new EmbedBuilder()
            .setTitle("Post Stats Results")
            .setColor("Blue");

        for (const key of Object.keys(results)) {
            let res = results[key]

            if(res.ok) {
                embed.addFields(
                    {
                        name: key,
                        value: `Successfully posted stats (${res.status})`,
                        inline: true
                    }
                )
            } else {
                let text = await res.text()
                ctx.client.logger.error("Error posting stats", { key, res, text })
                embed.addFields(
                    {
                        name: key,
                        value: `Status: \`${res.status}\`\n${res.statusText}. Check logs`,
                        inline: true
                    }
                )
            }
        }

        return FinalResponse.reply({
            embeds: [embed]
        })
    }
}

export default command;