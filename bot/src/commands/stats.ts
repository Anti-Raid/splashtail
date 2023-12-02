import { Colors, EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { BotStaffPerms, Command, FinalResponse } from "../core/client";
import { readFileSync } from "fs";
import { cpus, totalmem, freemem, release } from "os"
import { version as djsVersion } from "discord.js"
import { ContextReplyStatus } from "../core/context";
import { roundToTwo, uptimeToHuman, formatDate } from "../core/common/utils";
import { getServerCount, getShardCount, getUserCount } from "../core/common/counts";
  
const getCpuUsage = () => {
	// Take the first CPU, considering every CPUs have the same specs
	// and every NodeJS process only uses one at a time.
	const cpuList = cpus();
	if(!cpuList?.length) return 0;
	const cpu = cpuList[0];

	// Accumulate every CPU times values
	const total = Object.values(cpu.times).reduce(
		(acc: number, tv: number) => acc + tv, 0
	);

	// Normalize the one returned by process.cpuUsage() 
	// (microseconds VS miliseconds)
	const usage = process.cpuUsage();
	const currentCPUUsage = (usage.user + usage.system) / 1000;

	// Find out the percentage used for this specific CPU
	const perc = (currentCPUUsage / total) * 100;

	return Math.round(perc * 100) / 100;
}

let command: Command = {
    userPerms: [],
    botPerms: [],
    botStaffPerms: [],
    interactionData: new SlashCommandBuilder()
	.setName("stats")
	.setDescription("Shows bot stats")
	.addStringOption((type) =>
		type.setName("type")
		.setDescription("Which stats to show")
		.addChoices(
			{ name: "General Information", value: "info" },
			{ name: "System Information", value: "system" }
		)
	.setRequired(true)),
    execute: async (ctx) => {
		const type = ctx.interaction.options.getString("type") as string;

		switch (type) {
			case "info":
				await ctx.defer()

				let guildCount = 0
				let shardCount = 0
				let userCount = 0

				try {
					guildCount = await getServerCount(ctx.client)
					shardCount = await getShardCount(ctx.client)
					userCount = await getUserCount(ctx.client)
				} catch (err) {
					ctx.client.logger.error("Stats.GetCounts", err)
				}

				const embed = new EmbedBuilder()
					.setTitle(
						"Bot stats",
					)
					.setAuthor(
						{
							name: ctx.interaction.client.user.username,
							iconURL: ctx.interaction.client.user.displayAvatarURL(),
						}
					)
					.addFields([
						{
							name: "Name",
							value: ctx.interaction.client.user.username,
							inline: true,
						},
						{
							name: "ID",
							value: ctx.interaction.client.user.id,
							inline: true,
						},
						{
							name: "Ping",
							value: ctx.interaction.client.ws.ping + "ms",
							inline: true,
						},
						{
							name: "Uptime",
							value: uptimeToHuman(ctx.interaction.client.uptime),
							inline: true,
						},
						{
							name: "Servers",
							value: guildCount.toString(),
							inline: true,
						},
						{
							name: "Shards",
							value: shardCount.toString(),
							inline: true,
						},
						{
							name: "Cluster",
							value: `${ctx.client.clusterName} (${ctx.client.clusterId}) with ${ctx.client.clusterCount} clusters`,
							inline: true,
						},
						{
							name: "Users",
							value: userCount.toString(),
							inline: true,
						},
						{
							name: "Created At",
							value: formatDate(ctx.interaction.client.user.createdAt, "yyyy-MM-dd hh:mm:ss"),
							inline: true,
						},
						{
							name: "Last Restart",
							value: formatDate(ctx.interaction.client.readyAt, "yyyy-MM-dd hh:mm:ss"),
							inline: true,
						},
					])
					.setThumbnail(
						ctx.interaction.client.user.displayAvatarURL()
					)
					.setColor("Blurple");

				await ctx.reply({ embeds: [embed] });
				break;

			case "system":
				let memoryFree = freemem() / 1000000;
				let memoryTotal = totalmem() / 1000000;
				let memoryUsed = memoryTotal - memoryFree;
				let memoryUsage = (memoryUsed / memoryTotal) * 100;
				let cpuPercentage = getCpuUsage()

				let platform = process.platform;
				let osRelease = release();

				if(platform == "linux") {
					// Read /etc/os-release
					let osReleaseFile = readFileSync("/etc/os-release", {
						encoding: "utf-8"
					});

					let osReleaseFileLines = osReleaseFile.split("\n");

					for (const line of osReleaseFileLines) {
						if(line.startsWith("PRETTY_NAME=")) {
							osRelease = line.split("=")[1];
							break;
						}
					}
				}

				const embed2 = new EmbedBuilder()
				.setTitle(
					"System Information",
				)
				.setAuthor(
					{
						name: ctx.interaction.client.user.username,
						iconURL: ctx.interaction.client.user.displayAvatarURL(),
					}
				)
				.addFields([
					{
						name: "Operating System",
						value: `${platform} | ${osRelease}`,
						inline: true,
					},
					{
						name: `CPU`,
						value: `${cpuPercentage}%`,
						inline: true,
					},
					{
						name: "Memory",
						value: `**Total:** ${roundToTwo(memoryTotal)}MB\n**Used:** ${roundToTwo(memoryUsed)}MB\n**Free:** ${roundToTwo(memoryTotal - memoryUsed)}MB\n**Usage:** ${roundToTwo(memoryUsage)}%`,
						inline: true,
					},
					{
						name: "Discord.JS",
						value: `v${djsVersion}`,
						inline: true,
					},
					{
						name: "Node.JS",
						value: process.version,
						inline: true,
					},
				])
				.setThumbnail(
					ctx.interaction.client.user.displayAvatarURL()
				)
				.setColor(Colors.DarkRed);

			await ctx.reply({ embeds: [embed2] });
		}

		if(ctx.replyStatus == ContextReplyStatus.Pending) {
			return FinalResponse.reply({
				embeds: [
					new EmbedBuilder()
						.setColor("Orange")
						.setDescription(
							`Oops! We couldn't find any stats for that type.`
						),
				],
				fetchReply: true,
			});
		}

		return FinalResponse.dummy();
	}
}

export default command;