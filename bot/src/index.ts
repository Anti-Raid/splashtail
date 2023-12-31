// Packages
import { AntiRaid } from "./core/client";

// Get args from mewld
const args = process.argv.slice(2);

/**
				cmd = exec.Command(
					l.Config.Interp,
					l.Dir+"/"+l.Config.Module,
					mutils.ToPyListUInt64(i.Shards), // 0
					mutils.UInt64ToString(l.ShardCount), // 1
					strconv.Itoa(i.ClusterID), // 2
					cm.Name, // 3
					l.Dir, // 4
					strconv.Itoa(len(l.Map)), // 5
					state.Config.Meta.Proxy, // 6
					state.Config.Sites.API.Parse(), // 7
				)
*/
if (args.length < 8) {
	console.error("Usage: node . <shard count> <total shard count> <cluster id> <cluster name> <cluster directory> <no clusters> <proxy url> <api url>");
	console.error("Please ensure that the bot is being run via splashtail/mewld");
	process.exit(1);
}

const shards: number[] = JSON.parse(args[0]);
const shardCount: number = parseInt(args[1]);

if(!shards || !shardCount) {
	console.error("Invalid shard data");
	process.exit(1);
}

const clusterId = parseInt(args[2]);
const clusterName: string = args[3];
const clusterCount = parseInt(args[5]);

if(!clusterName || !clusterCount) {
	console.error("Invalid cluster data");
	process.exit(1);
}

const proxyUrl = args[6];
const apiUrl = args[7];

// Create Discord Client
const client = new AntiRaid(clusterId, clusterName, shards, shardCount, clusterCount, proxyUrl, apiUrl);

/*
// Guild member update event
client.on(Events.GuildMemberUpdate, async (oldMember, newMember) => {
	// Check if the member's nickname has changed
	if (oldMember.nickname !== newMember.nickname) {
		const guild = await getGuild(oldMember.guild.id);
		const embed = new EmbedBuilder()
			.setColor("Orange")
			.setDescription(
				`***${newMember.user.tag}*** has changed their nickname to **${newMember.nickname}**`
			)
			.setTimestamp();

		if (guild)
			client.channels.cache.get(guild.audit).send({ embeds: [embed] });
	}

	// Check if the member has been added or removed from any roles
	const addedRoles = newMember.roles.cache.filter(
		(role) => !oldMember.roles.cache.has(role.id)
	);
	const removedRoles = oldMember.roles.cache.filter(
		(role) => !newMember.roles.cache.has(role.id)
	);

	if (addedRoles.size > 0) {
		const guild = await getGuild(oldMember.guild.id);
		const embed = new EmbedBuilder()
			.setColor("Green")
			.setDescription(
				`***${
					newMember.user.tag
				}*** has been given the roles: \n**${addedRoles.map(
					(role) => `- ${role}`
				)}**`
			)
			.setTimestamp();

		if (guild)
			client.channels.cache.get(guild.audit).send({ embeds: [embed] });
	}

	if (removedRoles.size > 0) {
		const guild = getGuild(oldMember.guild.id);
		const embed = new EmbedBuilder()
			.setColor("Red")
			.setDescription(
				`***${
					newMember.user.tag
				}*** has has been removed from roles: \n**${removedRoles.map(
					(role) => `- ${role}`
				)}**`
			)
			.setTimestamp();

		if (guild)
			client.channels.cache.get(guild.audit).send({ embeds: [embed] });
	}
});

// Discord Message Events
client.on(Events.MessageUpdate, async (oldMessage, newMessage) => {
	// Ignore messages from other bots
	if (newMessage.author.bot) return;

	// Check if the message content has changed
	if (oldMessage.content !== newMessage.content) {
		const guild = await getGuild(oldMessage.guild.id);
		const embed = new EmbedBuilder()
			.setColor("Orange")
			.setDescription(
				`***${newMessage.author.tag}*** edited their message in *${
					newMessage.channel.name
				}*\n**Old Message:** \n> ${oldMessage.content.substr(
					0,
					1024
				)}\n**New Message:**\n> ${newMessage.content.substr(0, 1024)}`
			)
			.setTimestamp();

		if (guild)
			client.channels.cache.get(guild.audit).send({ embeds: [embed] });
	}
});

client.on(Events.MessageDelete, async (message) => {
	// Ignore messages from other bots
	if (message.author.bot) return;

	const guild = await getGuild(oldMember.guild.id);
	const embed = new EmbedBuilder()
		.setColor("Orange")
		.setDescription(
			`***${message.author.tag}*** has deleted their message in *${
				message.channel.name
			}*\n**Deleted Message: \n> ${message.content.substr(0, 1024)}`
		)
		.setTimestamp();

	if (guild) client.channels.cache.get(guild.audit).send({ embeds: [embed] });
});
*/

// Login to Discord
client.start();
