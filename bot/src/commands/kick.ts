import { EmbedBuilder, GuildMember, GuildMemberRoleManager, PermissionsBitField, Routes } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../core/common/guilds/auditor";
import sql from "../core/db";
import { channelPurger, parseDuration } from "../core/common/utils";
import { moderateUser } from "../core/common/guilds/mod";

let command: Command = {
    userPerms: [PermissionsBitField.Flags.KickMembers],
    botPerms: [PermissionsBitField.Flags.KickMembers],
    interactionData: new SlashCommandBuilder()
	.setName("kick")
	.setDescription("Kicks a user from the server.")
	.addUserOption((option) => option
		.setName("user")
		.setDescription("Who is the target user?")
		.setRequired(true)
    )
	.addStringOption((option) => option
		.setName("reason")
		.setDescription("Why are you kicking this user?")
        .setMaxLength(512)
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
            op: "kick",
            via: "cmd:kick",
            guildMember,
            reason,
            duration: 0,
            deleteMessagesTill,
        })
    }
}

export default command;