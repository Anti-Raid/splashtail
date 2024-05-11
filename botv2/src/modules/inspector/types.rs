/// The maximum number of mentions before the anti-everyone trigger is activated
pub const MAX_MENTIONS: u32 = 10;

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct TriggeredFlags: u32 {
        const NONE = 0;
        const ANTI_INVITE = 1 << 0;
        const ANTI_EVERYONE = 1 << 1;
        const MINIMUM_ACCOUNT_AGE = 1 << 2;
        const MAXIMUM_ACCOUNT_AGE = 1 << 3;
        const FAKE_BOT_DETECTION = 1 << 4;
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct DehoistOptions: i32 {
        const DISABLED = 1 << 0;
        const STRIP_NON_ASCII = 1 << 1;
        const STRIP_SIMILAR_REPEATING = 1 << 2;
    }
}