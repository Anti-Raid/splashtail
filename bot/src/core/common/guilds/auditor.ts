import postgres from "postgres"

/**
create table audit_logs (
      id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
      type TEXT NOT NULL,
      user_id TEXT NOT NULL,
      guild_id TEXT NOT NULL REFERENCES guilds(id) ON UPDATE CASCADE ON DELETE CASCADE,
      data JSONB NOT NULL,
      created_at TIMESTAMP NOT NULL DEFAULT NOW(),
      last_modified TIMESTAMP NOT NULL DEFAULT NOW()
)

create table guild_actions (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    type TEXT NOT NULL,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(id) ON UPDATE CASCADE ON DELETE CASCADE,
    data JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    audit_log_entry UUID NOT NULL REFERENCES audit_logs(id),
    expiry INTERVAL NOT NULL,
    last_modified TIMESTAMP NOT NULL DEFAULT NOW()
 );
 */

/**
 * Data to create an audit log event
 */
export interface CreateAuditLogEvent {
    /**
     * The type of event
     */
    type: string,
    /**
     * The user id who triggered the event
     */
    userId: string,
    /**
     * The guild id where the event was triggered
     */
    guildId: string,
    /**
     * Extra data of the event to be stored
     */
    data: { [key: string]: any }
}

/**
 * Adds an audit log event
 * 
 * @param event The event to add
 * @returns The ID of the added event
 */
export const addAuditLogEvent = async (sql: postgres.Sql<{}> | postgres.TransactionSql<{}>, event: CreateAuditLogEvent): Promise<string> => {
    let id =  await sql`
        INSERT INTO audit_logs ${sql(event, 'type', 'userId', 'guildId', 'data')} RETURNING id
    `
    return id[0].id
}

/**
 * Edits an audit log event
 * 
 * @param id The ID of the event to edit
 * @param event The event to edit
 */
export const editAuditLogEvent = async (sql: postgres.Sql<{}> | postgres.TransactionSql<{}>, id: string, event: CreateAuditLogEvent) => {
    await sql`
        UPDATE audit_logs SET ${sql(event, 'type', 'userId', 'guildId', 'data')} WHERE id = ${id}
    `
}


/**
 * Data to create a guild action
 */
export interface CreateGuildAction {
    /**
     * The type of action
     */
    type: string,
    /**
     * The user id who triggered the action
     */
    userId: string,
    /**
     * The guild id where the action was triggered
     */
    guildId: string,
    /**
     * Extra data of the action to be stored
     */
    data: { [key: string]: any },
    /**
     * Audit log entry ID
     */
    auditLogEntry: string,
    /**
     * The expiry of the action
     */
    expiry: string
}

/**
 * Adds a guild action
 * 
 * @param action The action to add
 * @returns The ID of the added action
 */
export const addGuildAction = async (sql: postgres.Sql<{}> | postgres.TransactionSql<{}>, action: CreateGuildAction): Promise<string> => {
    let fAction = {
        ...action,
        expiry: `INTERVAL '${action.expiry}'`
    }

    let id =  await sql`
        INSERT INTO guild_actions ${sql(fAction, 'type', 'userId', 'guildId', 'data', 'expiry', 'auditLogEntry')} RETURNING id
    `
    return id[0].id
}