import { EmbedBuilder, GuildMember, GuildMemberRoleManager, PermissionsBitField, Routes } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../core/common/guilds/auditor";
import sql from "../core/db";
import { channelPurger, parseDuration } from "../core/common/utils";
import { moderateUser } from "../core/common/guilds/mod";

let command: Command = {
    userPerms: [PermissionsBitField.Flags.ModerateMembers],
    botPerms: [PermissionsBitField.Flags.ModerateMembers],
    interactionData: new SlashCommandBuilder()
	.setName("timeout")
	.setDescription("Times out a user from the server.")
	.addUserOption((option) => option
		.setName("user")
		.setDescription("Who is the target user?")
		.setRequired(true)
    )
	.addStringOption((option) => option
		.setName("reason")
		.setDescription("Why are you timing out this user?")
        .setMaxLength(512)
	)
    .addStringOption((option) => option
        .setName("duration")
        .setDescription("How long should this timeout last with maximum of 28 days. Defaults to '7d'.")
    )
    .addStringOption((option) => option
        .setName("delete_messages_till")
        .setDescription("Delete messages until this many units prior. Defaults to 7d. Limited to 500 messages per channel.")
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
        const deleteMessagesTill = parseDuration(ctx.interaction.options.getString("delete_messages_till") || "7d")
        const duration = parseDuration(ctx.interaction.options.getString("duration") || "7d")

        // If duration is 0, send an error
        if(duration == 0) {
            return FinalResponse.reply({
                content: "You cannot timeout a user for this duration!",
                ephemeral: true,
            });
        }

        // Maximum timeout period is 28 days
        if(duration > 28*86400) {
            return FinalResponse.reply({
                content: "You cannot timeout a user for more than 28 days!",
                ephemeral: true,
            });
        }

        // Ensure deleteMessagesTill is less than 2 weeks
        if(deleteMessagesTill > 1209600) {
            return FinalResponse.reply({
                content: "You cannot delete messages older than 2 weeks!",
                ephemeral: true,
            });
        }

        await ctx.defer()

        return await moderateUser({
            ctx,
            op: "timeout",
            via: "cmd:timeout",
            guildMember,
            reason,
            duration,
            deleteMessagesTill,
        })
    }
}

export default command;