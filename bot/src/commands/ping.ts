import { EmbedBuilder } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";

let command: Command = {
    userPerms: [],
    botPerms: [],
    interactionData: new SlashCommandBuilder()
    .setName("ping")
	.setDescription("Pong!"),
    execute: async (ctx) => {
		const reply = await ctx.reply({
			embeds: [
				new EmbedBuilder()
					.setColor("Orange")
					.setDescription(
						`Checking Local Websocket Latency & Discord Interaction Roundtrip Latency...`
					),
			],
			fetchReply: true,
		});

		const interactionLatency = Math.round(
			reply.createdTimestamp - ctx.interaction.createdTimestamp
		);

        //await fetch(`https://api.instatus.com/v1/${ctx.client.config.instatus.page_id}/metrics/${ctx.client.config.instatus.metrics.}`)
		ctx.client.logger.info("ree", ctx.client.config.instatus);

        await ctx.edit({
			embeds: [
				new EmbedBuilder().setColor("Blue").addFields(
					{
						name: `Local Websocket Latency`,
						value: `\`${ctx.client.ws.ping}\`ms`,
						inline: true,
					},
					{
						name: `Discord Interaction Roundtrip Latency`,
						value: `\`${interactionLatency}\`ms`,
						inline: true,
					}
				),
			],
        });

        return FinalResponse.dummy()
    }
}

export default command;