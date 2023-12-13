import { EmbedBuilder, PermissionsBitField } from "discord.js";
import { AntiRaid, Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import sql from "../core/db";

let availableGuildCommandTypes = []
let guildCommandTypesIdCache = []

let command: Command = {
    userPerms: [PermissionsBitField.Flags.Administrator],
    botPerms: [],
    interactionData: async (client: AntiRaid) => {
		let guildChannelTypes = await sql`
			SELECT id, name, description FROM guild_channel_types
		`

		availableGuildCommandTypes = guildChannelTypes.map((type) => {
			return {
				name: type.name,
				value: type.id,
				description: type.description,
			}
		})

		guildCommandTypesIdCache = guildChannelTypes.map((type) => type.id)

		return new SlashCommandBuilder()
		.setName("settings")
		.setDescription("Configure the settings of the bot!")
		.addSubcommand((subcommand) => {
			subcommand
			.setName("channels")
			.setDescription("Configure the channels of the bot!")
			.addStringOption((option) => option
				.setName("type")
				.setDescription("Which type of channel to configure?")
				.addChoices(...availableGuildCommandTypes)
				.setRequired(true)
			)
			.addChannelOption((option) => option
				.setName("channel")
				.setDescription("Which channel to set?")
				.setRequired(true)
			)
			return subcommand
		})
	},
    execute: async (ctx) => {
		const subcommand = ctx.interaction.options.getSubcommand()

		switch (subcommand) {
			case "channels":
				const type = ctx.interaction.options.getString("type")
				const channel = ctx.interaction.options.getChannel("channel")

				// Ensure type is in availableGuildCommandTypes, using a cache to improve performance
				if(!guildCommandTypesIdCache.includes(type)) {
					return FinalResponse.reply({
						content: "This channel type is not yet implemented/support. Please contact support on our official discord server if you recieve this error",
						ephemeral: true,
					})
				}

				let typeData = availableGuildCommandTypes.find((t) => t.value === type)

				if(!typeData) {
					return FinalResponse.reply({
						content: "This channel type is not yet implemented/support. Please contact support on our official discord server if you recieve this error",
						ephemeral: true,
					})
				}

				// Defer the interaction as saving to the database can take a while
				await ctx.defer({
					ephemeral: true,
				})

				await sql`
					INSERT INTO guild_channels (guild_id, channel_id, channel_type)
					VALUES (${ctx.interaction.guildId}, ${channel.id}, ${type})
					ON CONFLICT (guild_id, channel_type) DO UPDATE
					SET channel_id = ${channel.id}
				`

				return FinalResponse.reply({
					content: `Successfully set the ${typeData?.name} (${type}) channel to ${channel}`,
					ephemeral: true,
				})
			default:
				return FinalResponse.reply({
					content: "This settings subcommand is not yet implemented. Please contact support on our official discord server if you recieve this error",
					ephemeral: true,
				})
		}
				
    }
}

export default command;