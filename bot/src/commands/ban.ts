import { EmbedBuilder, GuildMember, GuildMemberRoleManager, PermissionsBitField, Routes } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../core/common/guilds/auditor";
import sql from "../core/db";
import { parseDuration } from "../core/common/utils";
import { moderateUser } from "../core/common/guilds/mod";

let command: Command = {
    userPerms: [PermissionsBitField.Flags.BanMembers],
    botPerms: [PermissionsBitField.Flags.BanMembers],
    interactionData: new SlashCommandBuilder()
	.setName("ban")
	.setDescription("Bans a user from the server.")
	.addUserOption((option) => option
		.setName("user")
		.setDescription("Who is the target user?")
		.setRequired(true)
    )
	.addStringOption((option) => option
		.setName("reason")
		.setDescription("Why are you banning this user?")
        .setMaxLength(512)
	)
    .addStringOption((option) => option
        .setName("duration")
        .setDescription("How long should this ban last? Use '0' or leave blank for permanent. Defaults to permanent.")
    )
    .addStringOption((option) => option
        .setName("delete_messages_till")
        .setDescription("Delete messages until this many units prior. Defaults to 7d")
    ),
    execute: async (ctx) => {
        // @ts-expect-error
        const guildMember: GuildMember = ctx.interaction.options.getMember("user");
        if (!guildMember)
            return FinalResponse.reply({
                content: "This user is not an member of this guild.",
                ephemeral: true,
            });   
                    
        const reason = ctx.interaction.options.getString("reason");
        const duration = parseDuration(ctx.interaction.options.getString("duration"))
        const deleteMessagesTill = parseDuration(ctx.interaction.options.getString("delete_messages_till") || "7d")

        await ctx.defer()

        return await moderateUser({
            ctx,
            op: "ban",
            via: "cmd:ban",
            guildMember,
            reason,
            duration,
            deleteMessagesTill,
        })
    }
}

export default command;