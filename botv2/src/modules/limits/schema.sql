CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Stores the limits that are applied to a guild
CREATE TABLE limits__guild_limits (
    guild_id TEXT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT PRIMARY KEY DEFAULT uuid_generate_v4(),
    limit_name TEXT NOT NULL default 'Untitled',
    limit_type TEXT NOT NULL,
    limit_action TEXT NOT NULL,
    limit_per INTEGER NOT NULL,
    limit_time INTERVAL NOT NULL
);


-- Stores a list of user actions and which limits they have hit
-- A user action contributes to a limit
CREATE TABLE limits__user_actions (
    action_id TEXT PRIMARY KEY,
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL,
    limits_hit TEXT[] NOT NULL DEFAULT '{}'
);

-- Stores the past limits that have been applied in a guild
CREATE TABLE limits__past_hit_limits (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT NOT NULL REFERENCES limits(limit_id) ON DELETE CASCADE ON UPDATE CASCADE,
    cause TEXT[] NOT NULL DEFAULT '{}',
    notes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
