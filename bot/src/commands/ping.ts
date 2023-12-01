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
						`Checking Discord Websocket Latency & Discord Interaction Roundtrip Latency...`
					),
			],
			fetchReply: true,
		});

		const interactionLatency = Math.round(
			reply.createdTimestamp - ctx.interaction.createdTimestamp
		);

		reply.edit({
			embeds: [
				new EmbedBuilder().setColor("Blue").addFields(
					{
						name: `Discord Websocket Latency`,
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