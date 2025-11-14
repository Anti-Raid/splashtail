-- Proposed Unified Template 'RFC'/project
--
-- This project moves content state
-- to its own table called template_pool with guild attached templates
-- and listings in the shop referencing the pool.
--
-- This enables for new features such as:
-- - Directly publishing a guild template to the shop
-- without needing to duplicate content

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
);

CREATE INDEX template_pool_idx ON template_pool (id, owner_type, owner_id, language, created_at);

-- Template shop listings, these reference templates in the pool
CREATE TABLE template_shop_listings (
    template_pool_ref UUID PRIMARY KEY UNIQUE REFERENCES template_pool(id) ON UPDATE CASCADE ON DELETE CASCADE,
    short TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'denied'
    default_events TEXT[] NOT NULL DEFAULT '{}'::text[],
    default_allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX template_shop_idx ON template_shop (template_pool_ref, review_state, created_at);

-- Attached templates added to a guild/user
CREATE TABLE attached_templates(
    -- Owner data
    owner_type TEXT NOT NULL, -- 'user' or 'guild'
    owner_id TEXT NOT NULL,   -- user ID or guild ID

    -- Data
    template_pool_ref UUID NOT NULL REFERENCES template_pool(id) ON UPDATE CASCADE ON DELETE CASCADE,
    source TEXT NOT NULL, -- Source of how this template was attached (e.g., 'shop_listing', 'created', etc.)
    allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    events TEXT[] NOT NULL DEFAULT '{}'::text[],

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (owner_type, owner_id, template_pool_ref)
);

CREATE TABLE builtins_custom_attachments (
    -- Owner data
    owner_type TEXT NOT NULL, -- 'user' or 'guild'
    owner_id TEXT NOT NULL,   -- user ID or guild ID

    events TEXT[] NOT NULL DEFAULT '{}'::text[],

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (owner_type, owner_id)
)