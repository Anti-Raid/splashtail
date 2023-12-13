import { BotStaffPerms, Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";

let command: Command = {
    userPerms: [],
    botPerms: [],
    botStaffPerms: [BotStaffPerms.Owner],
    interactionData: new SlashCommandBuilder()
    .setName("reload")
    .addSubcommand((subcommand) => {
        subcommand
        .setName("command")
        .setDescription("Reloads a specific command!")
        .addStringOption((option) => option
            .setName("command")
            .setDescription("Which command to reload?")
            .setRequired(true)
        )
        .addBooleanOption((option) => {
            option
            .setName("allow_new")
            .setDescription("Allow new commands to be created?")
            .setRequired(false)

            return option
        })


        return subcommand
    })
	.setDescription("Reloads commands!"),
    execute: async (ctx) => {
        const subcommand = ctx.interaction.options.getSubcommand()

        switch (subcommand) {
            case "command":
                const commandName = ctx.interaction.options.getString("command")

                if(!commandName) {
                    return FinalResponse.reply({
                        content: "You must provide a command name!",
                        ephemeral: true,
                    })
                }

                if(!ctx.client.commands.has(commandName)) {
                    if(!ctx.interaction.options.getBoolean("allow_new")) {
                        return FinalResponse.reply({
                            content: "This command does not exist!",
                            ephemeral: true,
                        })
                    } else {
                        ctx.client.logger.info("Reload.Command", `Command ${commandName} does not exist, creating it due to allow_new being true`)
                    }
                }

                // Note that loadCommand updates the command cache, so we don't need to do that here
                await ctx.client.loadCommand(`${commandName}.js`)

                return FinalResponse.reply({
                    content: "Command reloaded!",
                    ephemeral: true,
                })
            default:
                return FinalResponse.reply({
                    content: "Unknown subcommand! Please contact a developer!",
                    ephemeral: true,
                })
        }
    }
}

export default command;