CREATE TABLE gitlogs__guilds (
    guild_id TEXT PRIMARY KEY NOT NULL REFERENCES guilds(id),
    banned BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE gitlogs__webhooks (
    id TEXT PRIMARY KEY NOT NULL,
    guild_id TEXT NOT NULL REFERENCES gitlogs__guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    comment TEXT NOT NULL, -- A comment to help identify the webhook
    secret TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE gitlogs__repos (
    id TEXT PRIMARY KEY NOT NULL,
    guild_id TEXT NOT NULL REFERENCES gitlogs__guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    webhook_id TEXT NOT NULL REFERENCES gitlogs__webhooks(id) ON DELETE CASCADE ON UPDATE CASCADE,
    repo_name TEXT NOT NULL,
    channel_id TEXT NOT NULL, -- Channel ID to post to
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE gitlogs__event_modifiers (
    id TEXT PRIMARY KEY NOT NULL,
    guild_id TEXT NOT NULL REFERENCES gitlogs__guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    webhook_id TEXT NOT NULL REFERENCES gitlogs__webhooks(id) ON DELETE CASCADE ON UPDATE CASCADE, -- Webhook to apply to
    repo_id TEXT REFERENCES gitlogs__repos(id) ON DELETE CASCADE ON UPDATE CASCADE, -- Optional, if not set, will assume all repos
    events TEXT[] NOT NULL DEFAULT '{}', -- Events to capture in this modifier
    blacklisted boolean not null default false, -- Whether or not these events are blacklisted or not
    whitelisted boolean not null default false, -- Whether or not only these events can be sent
    redirect_channel TEXT, -- Channel ID to redirect to, otherwise use default channel
    priority INTEGER NOT NULL -- Priority to apply the modifiers in, applied in descending order
);

create table gitlogs__webhook_logs (
    log_id text primary key not null,
    entries text[] not null default '{}',
    created_at timestamp not null default now() -- Not in original git-logs but its a default anyways
);