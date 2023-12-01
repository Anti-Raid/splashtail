import { EmbedBuilder, GuildMember, GuildMemberRoleManager, PermissionsBitField, Routes } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../core/common/guilds/auditor";
import sql from "../core/db";
import { parseDuration } from "../core/common/utils";

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
            
        // Ensure that the member can ban the target member
        if (guildMember.roles.highest.comparePositionTo((ctx.interaction.member.roles as GuildMemberRoleManager).highest) > 0) {
            return FinalResponse.reply({
                content: "You cannot ban this user as they have a higher role than you.",
                ephemeral: true,
            });
        }
        
        const reason = ctx.interaction.options.getString("reason");
        const duration = parseDuration(ctx.interaction.options.getString("duration"))
        const deleteMessagesTill = parseDuration(ctx.interaction.options.getString("delete_messages_till") || "7d")

        await ctx.defer()

        let disallowDM = false;
        await sql.begin(async sql => {
            let auditLogEntry = await addAuditLogEvent(sql, {
                type: "ban",
                userId: ctx.interaction.user.id,
                guildId: ctx.interaction.guild.id,
                data: {
                    "status": "pending",
                    "target_id": guildMember.id,
                    "via": "cmd:ban",
                    "duration": duration,
                    "delete_messages_till": deleteMessagesTill,
                    "reason": reason
                }
            })
    
            try {
                const embed = new EmbedBuilder()
                    .setColor("Red")
                    .setTitle(`Banned from ${ctx.interaction.guild.name}`)
                    .setDescription(reason ?? "No reason specified.")
                    .setAuthor({
                        name: ctx.client.user.displayName,
                        iconURL: ctx.client.user.displayAvatarURL(),
                    })
                    .setTimestamp();
    
                await guildMember.send({ embeds: [embed] });
            } catch (error) {
                disallowDM = true;
            }     
            
            try {
                await guildMember?.ban({ 
                    reason: `${ctx.interaction.user.username} [${ctx.interaction.user.id}]: ${reason ?? "No reason specified."}`,
                    deleteMessageSeconds: deleteMessagesTill,
                });    
            } catch (err) {
                await editAuditLogEvent(sql, auditLogEntry, {
                    type: "ban",
                    userId: ctx.interaction.user.id,
                    guildId: ctx.interaction.guild.id,
                    data: {
                        "status": "failed",
                        "error": err.message,
                        "target_id": guildMember.id,
                        "via": "cmd:ban",
                        "duration": duration,
                        "delete_messages_till": deleteMessagesTill,
                        "reason": reason
                    }
                })    
                return FinalResponse.reply({
                    content: `Failed to ban **${guildMember.user.tag}**. Error: \`${err.message}\``,
                    ephemeral: true,
                });
            }

            await editAuditLogEvent(sql, auditLogEntry, {
                type: "ban",
                userId: ctx.interaction.user.id,
                guildId: ctx.interaction.guild.id,
                data: {
                    "status": "success",
                    "target_id": guildMember.id,
                    "via": "cmd:ban",
                    "duration": duration,
                    "delete_messages_till": deleteMessagesTill,
                    "reason": reason
                }
            })
    
            if(duration > 0) {
                await addGuildAction(sql, {
                    type: "unban",
                    userId: guildMember.id,
                    guildId: ctx.interaction.guild.id,
                    auditLogEntry,
                    expiry: `${duration} seconds`,
                    data: {
                        "via": "cmd:ban",
                        "duration": duration,
                        "reason": reason
                    }
                })   
            }    
        })

        return FinalResponse.reply(
            {
                embeds: [
                    new EmbedBuilder()
                        .setColor("Green")
                        .setDescription(
                            `Banned **${guildMember.user.tag}**${
                                disallowDM ? " (DMs disabled)" : ""
                            }`
                        )
                ]
            }
        )
    }
}

export default command;