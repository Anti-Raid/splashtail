import { EmbedBuilder, GuildMember, GuildMemberRoleManager } from "discord.js";
import { AntiRaid, FinalResponse } from "../../client";
import { addAuditLogEvent, addGuildAction, editAuditLogEvent } from "../guilds/auditor";
import sql from "../../db";
import { channelPurger, parseDuration } from "../utils";
import { CommandContext } from "../../context";

export type ModerationOp = "ban" | "kick" | "timeout";

const opPastParticiple = (op: ModerationOp) => {
    switch(op) {
        case "ban": return "banned"
        case "kick": return "kicked"
        case "timeout": return "timed out"
    }
}

export interface ModeratorUser {
    /**
     * ID of the user who performed this action
     */
    id: string;
    /**
     * Username of the user who performed this action
     */
    username: string;
}

export interface PerformModeration {
    /**
     * The bot object
     */
    client: AntiRaid;
    /**
     * The operation to perform
     */
    op: ModerationOp;
    /**
     * The guild member to perform this operation on
     */
    guildMember: GuildMember;
    /**
     * By whom was this operation performed
     */
    by?: ModeratorUser;
    /**
     * The method that was used to perform this operation
     */
    via: string;
    /**
     * The reason for this moderation action
     */
    reason: string;
    /**
     * How long this moderation action should last
     */
    duration: number;
    /**
     * Till what time should messages be deleted
     */
    deleteMessagesTill: number;
    /**
     * The audit log entry for this moderation action
     */
    auditLogEntry: string;
}

export enum PerformModerationState {
    Success,
    DeleteMessagesFailed,
}

export interface PerformModerationResult {
    state: PerformModerationState;
    member: GuildMember;
    error?: Error;
}

const performModeration = async ({ client, op, guildMember, by, via, reason, duration, deleteMessagesTill, auditLogEntry }: PerformModeration): Promise<PerformModerationResult> => {
    // Ban makes this really easy
    let m: GuildMember;
    switch (op) {
        case "ban":
            m = await guildMember?.ban({ 
                reason: `${by.username} [${by.id}]: ${reason ?? "No reason specified."}`,
                deleteMessageSeconds: deleteMessagesTill,
            }); 
            
            if(duration > 0) {
                await addGuildAction(sql, {
                    type: "unban",
                    userId: guildMember.id,
                    guildId: guildMember.guild.id,
                    auditLogEntry,
                    expiry: `${duration} seconds`,
                    data: {
                        "via": via,
                        "duration": duration,
                        "reason": reason
                    }
                })   
            }    

            return {
                state: PerformModerationState.Success,
                member: m
            }
        case "kick":     
            m = await guildMember?.kick(`${by.username} [${by.id}]: ${reason ?? "No reason specified."}`);    

            try {
                if(deleteMessagesTill > 0) {
                    let channels = await guildMember.guild.channels.fetch()

                    // Turn channels into a GuildChannel[]
                    let channelsArr = channels.map((channel) => channel)

                    await channelPurger(client, channelsArr, {
                        tillInterval: deleteMessagesTill,
                        memberIds: [guildMember.id]
                    })
                }
            } catch (err) {
                return {
                    state: PerformModerationState.DeleteMessagesFailed,
                    member: m,
                    error: err
                }
            }

            return {
                state: PerformModerationState.Success,
                member: m
            }
        case "timeout":
            m = await guildMember?.timeout(duration * 1000, `${by.username} [${by.id}]: ${reason ?? "No reason specified."}`);    

            try {
                if(deleteMessagesTill > 0) {
                    let channels = await guildMember.guild.channels.fetch()

                    // Turn channels into a GuildChannel[]
                    let channelsArr = channels.map((channel) => channel)

                    await channelPurger(client, channelsArr, {
                        tillInterval: deleteMessagesTill,
                        memberIds: [guildMember.id]
                    })
                }
            } catch (err) {
                return {
                    state: PerformModerationState.DeleteMessagesFailed,
                    member: m,
                    error: err
                }
            }
        default:
            throw new Error("Invalid moderation operation")
    }
}

export interface ModerateUser {
    /**
     * The context of the command
     */
    ctx: CommandContext;
    /**
     * The operation to perform
     */
    op: ModerationOp;
    /**
     * The method that was used to perform this operation
     */
    via: string;
    /**
     * The guild member to perform this operation on
     */
    guildMember: GuildMember;
    /**
     * The reason for this moderation action
     */
    reason: string;
    /**
     * How long this moderation action should last
     */
    duration: number;
    /**
     * Till what time should messages be deleted
     */
    deleteMessagesTill: number;
}

export const moderateUser = async ({ ctx, op, via, guildMember, reason, duration, deleteMessagesTill }: ModerateUser) => {        
    // Ensure that the member can ban the target member
    if (guildMember.roles.highest.comparePositionTo((ctx.interaction.member.roles as GuildMemberRoleManager).highest) > 0) {
        return FinalResponse.reply({
            content: `You cannot moderate (${op}) this user as they have a higher role than you.`,
            ephemeral: true,
        });
    }

    // Ensure thatt the bot can ban the target member
    let botMember: GuildMember;

    // First from cache
    botMember = ctx.interaction.guild.members.cache.get(ctx.client.user.id) as GuildMember;

    if(!botMember) {
        // Then from API
        botMember = await ctx.interaction.guild.members.fetch(ctx.client.user.id) as GuildMember;
    }

    if (guildMember.roles.highest.comparePositionTo(botMember.roles.highest) > 0) {
        return FinalResponse.reply({
            content: `I cannot moderate (${op}) this user as they have a higher role than me.`,
            ephemeral: true,
        });
    }
    
    let disallowDM = false;
    let errResp: FinalResponse | undefined;
    let pmr: PerformModerationResult | undefined;
    await sql.begin(async sql => {
        let auditLogEntry = await addAuditLogEvent(sql, {
            type: op,
            userId: ctx.interaction.user.id,
            guildId: ctx.interaction.guild.id,
            data: {
                "status": "pending",
                "target_id": guildMember.id,
                "via": via,
                "duration": duration,
                "delete_messages_till": deleteMessagesTill,
                "reason": reason
            }
        })

        try {
            const embed = new EmbedBuilder()
                .setColor("Red")
                .setTitle(`${opPastParticiple(op)} from ${ctx.interaction.guild.name}`)
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
            pmr = await performModeration({
                client: ctx.client,
                op,
                guildMember,
                by: {
                    id: ctx.interaction.user.id,
                    username: ctx.interaction.user.username
                },
                via,
                reason,
                duration,
                deleteMessagesTill,
                auditLogEntry
            });    
        } catch (err) {
            ctx.client.logger.error("ModerateUser", `Failed to ${op} user ${guildMember.id} in guild ${ctx.interaction.guild.id}: ${err.message}`)
            await editAuditLogEvent(sql, auditLogEntry, {
                type: op,
                userId: ctx.interaction.user.id,
                guildId: ctx.interaction.guild.id,
                data: {
                    "status": "failed",
                    "error": err.message,
                    "target_id": guildMember.id,
                    "via": via,
                    "duration": duration,
                    "delete_messages_till": deleteMessagesTill,
                    "reason": reason
                }
            })    

            errResp = FinalResponse.reply({
                content: `Failed to ${op} **${guildMember.user.tag}**. Error: \`${err.message}\``,
                ephemeral: true,
            });
            return
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
    })

    if(errResp) {
        return errResp
    }

    switch (pmr.state) {
        case PerformModerationState.DeleteMessagesFailed:
            return FinalResponse.reply({
                content: `Failed to delete messages for **${guildMember.user.tag}**. Error: \`${pmr.error?.message}\``,
                ephemeral: true,
            });
    }

    return FinalResponse.reply(
        {
            embeds: [
                new EmbedBuilder()
                    .setColor("Green")
                    .setDescription(
                        `${opPastParticiple(op)} **${guildMember.user.tag}**${
                            disallowDM ? " (DMs disabled)" : ""
                        }`
                    )
            ]
        }
    )
}