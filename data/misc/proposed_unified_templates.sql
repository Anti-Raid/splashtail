-- Proposed Unified Template 'RFC'/project
--
-- While not as major as v2 unified templates, it is much simpler to implement
CREATE TABLE template_shop_listings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    short TEXT NOT NULL,
    review_state TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'denied'
    default_events TEXT[] NOT NULL DEFAULT '{}'::text[],
    default_allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    
    -- Content VFS
    language TEXT NOT NULL DEFAULT 'luau',
    content JSONB NOT NULL, -- The actual template content

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX template_shop_idx ON template_shop (id, review_state, created_at);

-- Attached templates added to a guild/user
CREATE TABLE attached_templates(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Owner data
    owner_type TEXT NOT NULL, -- 'user' or 'guild'
    owner_id TEXT NOT NULL,   -- user ID or guild ID

    -- Data
    source TEXT NOT NULL, -- Source of how this template was attached ('builtins', 'shop' or 'custom')

    -- data custom
    name TEXT,
    language TEXT,
    content JSONB, -- The actual template content

    -- data shop
    shop_ref UUID REFERENCES template_shop_listings(id) ON UPDATE CASCADE ON DELETE CASCADE,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    allowed_caps TEXT[] NOT NULL DEFAULT '{}'::text[],
    events TEXT[] NOT NULL DEFAULT '{}'::text[],
    state TEXT NOT NULL DEFAULT 'active' -- 'active', 'paused' or 'suspended'
);

ALTER TABLE 
attached_templates
ADD 
  CONSTRAINT attached_templates_disc_union CHECK (
    (
      attached_templates.source = 'custom' 
      AND attached_templates.name IS NOT NULL 
      AND attached_templates.language IS NOT NULL
      AND attached_templates.content IS NOT NULL
      AND attached_templates.shop_ref IS NULL
    ) 
    OR (
      attached_templates.source = 'builtins' 
      AND attached_templates.name IS NULL 
      AND attached_templates.language IS NULL
      AND attached_templates.content IS NULL
      AND attached_templates.shop_ref IS NULL
    )
    OR (
      attached_templates.source = 'shop' 
      AND attached_templates.name IS NULL 
      AND attached_templates.language IS NULL
      AND attached_templates.content IS NULL
      AND attached_templates.shop_ref IS NOT NULL
    )
  );
