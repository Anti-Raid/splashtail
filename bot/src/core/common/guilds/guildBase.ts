import { Guild } from "discord.js"
import sql from "../../db"

/**
 * Creates a guild along with a team with the name of the guild
 */
export const createGuildIfNotExists = async (guild: Guild) => {
    // Check if guild already exists
    let guildExists = await sql`SELECT COUNT(*) FROM guilds WHERE id = ${guild.id}`

    if(guildExists[0].count > 0) {
        return
    }

    await sql.begin(async sql => {
        await sql`
            INSERT INTO guilds (id) VALUES (${guild.id})
        `
    })
}

