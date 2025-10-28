-- The template pool which stores all templates
CREATE TABLE template_pool (
    -- Identifier for the template in the pool
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,

    -- Owner data
    owner_type TEXT NOT NULL, -- 'user' or 'guild'
    owner_id TEXT NOT NULL,   -- user ID or guild ID

    -- Content VFS
    language TEXT NOT NULL DEFAULT 'luau',
    content JSONB NOT NULL, -- The actual template content

    -- Key metadata
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    state TEXT NOT NULL DEFAULT 'active', -- 'active', 'paused', 'suspended'
);

CREATE INDEX template_pool_idx ON template_pool (id, owner_type, owner_id, language, created_at, state);

-- Template shop listings, these reference templates in the pool
CREATE TABLE template_shop_listings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_pool_ref UUID NOT NULL UNIQUE REFERENCES template_pool(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    review_state TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'denied'
    default_events TEXT[] NOT NULL DEFAULT '{}'::text[],
    default_allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX template_shop_idx ON template_shop (id, template_pool_ref, review_state, created_at);

-- Guild owned templates
CREATE TABLE guild_templates(
    guild_id TEXT NOT NULL,
    template_pool_ref UUID NOT NULL REFERENCES template_pool(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    events TEXT[] NOT NULL DEFAULT '{}'::text[],
    PRIMARY KEY (guild_id, template_pool_ref)
);
