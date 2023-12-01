import { EmbedBuilder, GuildMember, GuildMemberRoleManager, PermissionsBitField, Routes } from "discord.js";
import { Command, FinalResponse } from "../core/client";
import { SlashCommandBuilder } from "@discordjs/builders";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../core/common/guilds/auditor";
import sql from "../core/db";
import { channelPurger, parseDuration } from "../core/common/utils";

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
            
        // Ensure that the member can kick the target member
        if (guildMember.roles.highest.comparePositionTo((ctx.interaction.member.roles as GuildMemberRoleManager).highest) > 0) {
            return FinalResponse.reply({
                content: "You cannot kick this user as they have a higher role than you.",
                ephemeral: true,
            });
        }
        
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

        let disallowDM = false;
        await sql.begin(async sql => {
            let auditLogEntry = await addAuditLogEvent(sql, {
                type: "kick",
                userId: ctx.interaction.user.id,
                guildId: ctx.interaction.guild.id,
                data: {
                    "status": "pending",
                    "target_id": guildMember.id,
                    "via": "cmd:kick",
                    "delete_messages_till": deleteMessagesTill,
                    "reason": reason
                }
            })
    
            try {
                const embed = new EmbedBuilder()
                    .setColor("Red")
                    .setTitle(`Kicked from ${ctx.interaction.guild.name}`)
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
                await guildMember?.kick(`${ctx.interaction.user.username} [${ctx.interaction.user.id}]: ${reason ?? "No reason specified."}`);    
            } catch (err) {
                await editAuditLogEvent(sql, auditLogEntry, {
                    type: "kick",
                    userId: ctx.interaction.user.id,
                    guildId: ctx.interaction.guild.id,
                    data: {
                        "status": "failed",
                        "error": err.message,
                        "target_id": guildMember.id,
                        "via": "cmd:kick",
                        "delete_messages_till": deleteMessagesTill,
                        "reason": reason
                    }
                })    
                return FinalResponse.reply({
                    content: `Failed to kick **${guildMember.user.tag}**. Error: \`${err.message}\``,
                    ephemeral: true,
                });
            }

            try {
                if(deleteMessagesTill > 0) {
                    let channels = await ctx.interaction.guild.channels.fetch()

                    // Turn channels into a GuildChannel[]
                    let channelsArr = channels.map((channel) => channel)

                    await channelPurger(ctx, channelsArr, {
                        tillInterval: deleteMessagesTill,
                        memberIds: [guildMember.id]
                    })
                }
            } catch (err) {
                await editAuditLogEvent(sql, auditLogEntry, {
                    type: "kick",
                    userId: ctx.interaction.user.id,
                    guildId: ctx.interaction.guild.id,
                    data: {
                        "status": "partially_failed:deleteMessagesTill",
                        "error": err.message,
                        "target_id": guildMember.id,
                        "via": "cmd:kick",
                        "delete_messages_till": deleteMessagesTill,
                        "reason": reason
                    }
                })    
                return FinalResponse.reply({
                    content: `Kick succeeded but failed to delete messages from **${guildMember.user.tag}**. Error: \`${err.message}\``,
                    ephemeral: true,
                });
            }

            await editAuditLogEvent(sql, auditLogEntry, {
                type: "kick",
                userId: ctx.interaction.user.id,
                guildId: ctx.interaction.guild.id,
                data: {
                    "status": "success",
                    "target_id": guildMember.id,
                    "via": "cmd:kick",
                    "delete_messages_till": deleteMessagesTill,
                    "reason": reason
                }
            })
        })

        return FinalResponse.reply(
            {
                embeds: [
                    new EmbedBuilder()
                        .setColor("Green")
                        .setDescription(
                            `Kicked **${guildMember.user.tag}**${
                                disallowDM ? " (DMs disabled)" : ""
                            }`
                        )
                ]
            }
        )
    }
}

export default command;