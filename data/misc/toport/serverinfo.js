const { SlashCommandBuilder } = require("@discordjs/builders");
const { CommandInteraction } = require("discord.js");

module.exports = {
	data: new SlashCommandBuilder()
		.setName("serverinfo")
		.setDescription("Shows info about this server"),
	/**
	 *
	 * @param {CommandInteraction} interaction
	 */
	async execute(client, interaction, database) {
		const embed = new client.EmbedBuilder()
			.setThumbnail(interaction.guild.iconURL({ dynamic: true }))
			.setColor(interaction.member.displayHexColor ?? "Aqua")
			.setAuthor({
				name: `Server Info for ${interaction.guild.name} (${interaction.guild.nameAcronym})`,
				iconURL: interaction.user.displayAvatarURL({
					dynamic: true,
				}),
			})
			.addFields([
				{
					name: `Name`,
					value: `${limit(interaction.guild.name)}`,
					inline: true,
				},
				{
					name: `Name Acronym`,
					value: `${interaction.guild.nameAcronym}`,
					inline: true,
				},
				{
					name: `AFK Channel`,
					value: `<#${interaction.guild.afkChannel.id}>`,
					inline: true,
				},
				{
					name: `Total Bans`,
					value: `${
						interaction.guild.bans.cache.size ?? "Not Cached or 0"
					}`,
					inline: true,
				},
				{
					name: `Total Channels`,
					value: `${
						interaction.guild.channels.cache.size ??
						"Not Cached or 0"
					}`,
					inline: true,
				},
				{
					name: `Created At`,
					value: `<t:${interaction.guild.createdTimestamp}:F>`,
					inline: true,
				},
				{
					name: `Emojis`,
					value: `${
						interaction.guild.emojis.cache.size ?? "Not Cached or 0"
					}`,
					inline: true,
				},
				{
					name: `Server Features`,
					value: `${
						limit(
							interaction.guild.features
								.map((s) => {
									const string = [];
									const words = s
										.replaceAll("_", " ")
										.toLowerCase()
										.split(" ");
									words.map((word) => {
										string.push(
											word.charAt(0).toUpperCase() +
												word.slice(1)
										);
									});
									return string.join(" ");
								})
								.join(", ")
						) ?? "Couldn't Fetch Features"
					}`,
					inline: false,
				},
				{
					name: `Server Id`,
					value: `${interaction.guild.id}`,
					inline: true,
				},
				{
					name: `Member Count`,
					value: `${
						interaction.guild.memberCount ?? "Not Cached or 0"
					}`,
					inline: true,
				},
				{
					name: `Owner`,
					value: `${
						limit(
							interaction.client.users.cache.get(
								interaction.guild.ownerId
							).tag
						) ?? "Couldn't cache Server Owner's Username"
					}`,
					inline: true,
				},
				{
					name: `Roles (${
						interaction.guild.roles.cache.size ?? "0"
					})`,
					value: `> ${limit(
						interaction.guild.roles.cache
							.first(15)
							.filter((s) => s.name !== "@everyone")
							.sort((a, b) => b.position - a.position)
							.map((r) => r)
							.join(", ")
					)}`,
					inline: false,
				},
			]);

		await interaction.reply({
			embeds: [embed],
		});
	},
};

const limit = (value) => {
	let max_chars = 700;
	let i;

	if (value.length > max_chars) i = value.substr(0, max_chars);
	else i = value;

	return i;
};
