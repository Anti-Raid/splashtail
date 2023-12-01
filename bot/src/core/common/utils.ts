import { GuildChannel } from "discord.js";
import { CommandContext } from "../context";

export const parseDuration = (duration: string | undefined): number => {
    if(!duration) {
        return 0
    }

    if(duration == "0") {
        return 0
    }

    duration = duration.replaceAll(" ", "")

	var units = {
        // Days
        'days': 86400,
        'day': 86400,
        'd': 86400,
        // Hours
        'hours': 3600,
        'hour': 3600,
        'hrs': 3600,
        'hr': 3600,
        'h': 3600, 
        // Minutes
        'minutes': 60,
        'minute': 60,
        'mins': 60,
        'min': 60,
        'm': 60, 
        // Seconds
        'seconds': 1,
        'second': 1,
        'secs': 1,
        'sec': 1,
        's': 1
    };

    let seconds = 0;

    for (const [key, value] of Object.entries(units)) {
        let regex = new RegExp(`([0-9]+)${key}`, "i")
        let match = duration.match(regex)

        if(match) {
            let amount = parseInt(match[1])
            seconds += amount * value
        }
    }

	return seconds;
}

/**
 * Purges a channel of messages
 */
export interface ChannelPurgeOptions {
    /**
     * The members to purge messages from
     */
    memberIds?: string[]
    /**
     * Till how many seconds ago should messages be deleted. Is required
     */
    tillInterval: number
}

export const channelPurger = async (ctx: CommandContext, channels: GuildChannel[], opts: ChannelPurgeOptions) => {
    if(!opts.tillInterval) {
        throw new Error("tillInterval is required")
    }

    for (const channel of channels) {
        if(!channel.isTextBased()) {
            continue
        }

        let isNotDone = false
        let tries = 0
        let currentMessage = undefined    

        while (!isNotDone && tries < 5) {
            let messages = await channel.messages.fetch({ 
                limit: 100,
                ...(currentMessage ? { before: currentMessage.id } : {})
            })

            if(messages.size < 100) {
                isNotDone = true
            }

            let messagesToDelete = messages.filter(message => {
                // Basic till interval check
                if(message.createdTimestamp > Date.now() - opts.tillInterval * 1000) {
                    return false
                }

                let conditions: { [key: string]: () => boolean} = {
                    "memberIds": () => opts.memberIds.includes(message.author.id)
                }

                // Check if the message matches all conditions
                for (const [key, condition] of Object.entries(conditions)) {
                    if(opts[key] && !condition()) {
                        return false
                    }
                }
            })
            
            try {
                await channel.bulkDelete(messagesToDelete)
            } catch (err) {
                ctx.client.logger.error(`Failed to delete messages in channel ${channel.id} in guild ${ctx.interaction.guild.id}. Error: ${err.message}`)
            }

            // Now set currentMessage to the message with the oldest timestamp
            currentMessage = messages.sort((a, b) => a.createdTimestamp - b.createdTimestamp).first()
        }
    }
}