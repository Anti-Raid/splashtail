const { SlashCommandBuilder } = require("@discordjs/builders");
const { Client, CommandInteraction, ChannelType } = require("discord.js");

module.exports = {
	data: new SlashCommandBuilder()
		.setName("slowmode")
		.setDescription("Sets a slowmode in the specified channel.")
		.addChannelOption((option) =>
			option
				.addChannelTypes(ChannelType.GuildText)
				.setName("channel")
				.setDescription("The channel to set the slowmode in.")
				.setRequired(true)
		)
		.addIntegerOption((option) =>
			option
				.setName("time")
				.setDescription("Time in seconds for the slowmode.")
				.setRequired(true)
		),
	/**
	 *
	 * @param {Client} client
	 * @param {CommandInteraction} interaction
	 * @returns
	 */
	async execute(client, interaction, database) {
		const channel = interaction.options.getChannel("channel");
		const time = interaction.options.getInteger("time");

		if (time <= 21600) {
			await channel.setRateLimitPerUser(time);
			await interaction.reply(`Slowmode set to ${time} seconds.`);
		} else
			return interaction.reply({
				content: `> Whoops! You cannot set a higher slow mode of this channel! Maximum is 6 hours (21600 seconds).`,
				ephemeral: true,
			});
	},
};
