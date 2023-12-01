import { EmbedBuilder, Routes } from "discord.js";
import { BotStaffPerms, Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";

let command: Command = {
    userPerms: [],
    botPerms: [],
    botStaffPerms: [BotStaffPerms.Owner],
    interactionData: new SlashCommandBuilder()
    .setName("deploy")
	.setDescription("Deploys commands to the mothership!"),
    execute: async (ctx) => {
        // Get the global commands
        let globalCommands = []
        let guildOnlyCommands = []

        for (const [id, command] of ctx.client.commands) {
            if(command?.botStaffPerms?.includes(BotStaffPerms.Owner)) {
                guildOnlyCommands.push(command.interactionData.toJSON())
            } else {
                globalCommands.push(command.interactionData.toJSON())
            }
        }

        await ctx.reply({
            embeds: [
                new EmbedBuilder()
                    .setColor("Orange")
                    .setDescription(
                        `Commands Deploying...`
                    )
                    .addFields(
                        {
                            name: "Global Commands",
                            value: `\`\`\`json\n${globalCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        },
                        {
                            name: "Guild Only Commands",
                            value: `\`\`\`json\n${guildOnlyCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        }
                    ),
            ],
        })

        await ctx.client.rest.put(Routes.applicationCommands(ctx.client.application.id), { body: globalCommands })
        await ctx.client.rest.put(Routes.applicationGuildCommands(ctx.client.application.id, ctx.client.config.servers.main), { body: guildOnlyCommands })

        await ctx.edit({
            embeds: [
                new EmbedBuilder()
                    .setColor("Orange")
                    .setDescription(
                        `Commands Deployed...`
                    )
                    .addFields(
                        {
                            name: "Global Commands",
                            value: `\`\`\`json\n${globalCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        },
                        {
                            name: "Guild Only Commands",
                            value: `\`\`\`json\n${guildOnlyCommands.map(c => c.name)}\`\`\``,
                            inline: true
                        }
                    ),
            ],
        })

        return FinalResponse.dummy()
    }
}

export default command;