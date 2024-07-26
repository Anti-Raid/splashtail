pub mod embed_limits {
    pub const EMBED_TITLE_LIMIT: usize = 256;
    pub const EMBED_DESCRIPTION_LIMIT: usize = 4096;
    pub const EMBED_MAX_COUNT: usize = 10;
    pub const EMBED_FIELDS_MAX_COUNT: usize = 25;
    pub const EMBED_FIELD_NAME_LIMIT: usize = 256;
    pub const EMBED_FIELD_VALUE_LIMIT: usize = 1024;
    pub const EMBED_FOOTER_TEXT_LIMIT: usize = 2048;
    pub const EMBED_AUTHOR_NAME_LIMIT: usize = 256;
    pub const EMBED_TOTAL_LIMIT: usize = 6000;
}

pub mod message_limits {
    pub const MESSAGE_CONTENT_LIMIT: usize = 2000;
}
