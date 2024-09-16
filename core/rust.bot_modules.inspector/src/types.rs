/// The maximum number of mentions before the anti-everyone trigger is activated
pub const MAX_MENTIONS: u32 = 10;

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct TriggeredFlags: i32 {
        const NONE = 0;
        const ANTI_INVITE = 1 << 0;
        const ANTI_EVERYONE = 1 << 1;
        const MINIMUM_ACCOUNT_AGE = 1 << 2;
        const MAXIMUM_ACCOUNT_AGE = 1 << 3;
        const FAKE_BOT_DETECTION = 1 << 4;
    }
}

impl std::fmt::Display for TriggeredFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        for flag in self.iter() {
            let f = match flag {
                TriggeredFlags::NONE => "None",
                TriggeredFlags::ANTI_INVITE => "Anti Invite",
                TriggeredFlags::ANTI_EVERYONE => "Anti Everyone",
                TriggeredFlags::MINIMUM_ACCOUNT_AGE => "Minimum Account Age",
                TriggeredFlags::MAXIMUM_ACCOUNT_AGE => "Maximum Account Age",
                TriggeredFlags::FAKE_BOT_DETECTION => "Fake Bot Detection",
                _ => "Unknown",
            };

            flags.push(f);
        }

        write!(f, "{}", flags.join(", "))
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct DehoistOptions: i32 {
        const DISABLED = 1 << 0;
        const STRIP_SPECIAL_CHARS_STARTSWITH = 1 << 1;
        const STRIP_SPECIAL_CHARS_CONTAINS = 1 << 2;
        const STRIP_NON_ASCII = 1 << 3;
        const STRIP_SIMILAR_REPEATING = 1 << 4;
    }
}

impl std::fmt::Display for DehoistOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        for flag in self.iter() {
            let f = match flag {
                DehoistOptions::DISABLED => "Disabled",
                DehoistOptions::STRIP_SPECIAL_CHARS_STARTSWITH => {
                    "Strip Special Chars (Startswith)"
                }
                DehoistOptions::STRIP_SPECIAL_CHARS_CONTAINS => "Strip Special Chars (Contains)",
                DehoistOptions::STRIP_NON_ASCII => "Strip Non-ASCII",
                DehoistOptions::STRIP_SIMILAR_REPEATING => "Strip Similar Repeating",
                _ => "Unknown",
            };

            flags.push(f);
        }

        write!(f, "{}", flags.join(", "))
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct GuildProtectionOptions: i32 {
        const DISABLED = 1 << 0;
        const NAME = 1 << 1;
        const ICON = 1 << 2;
    }
}

impl std::fmt::Display for GuildProtectionOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        for flag in self.iter() {
            let f = match flag {
                GuildProtectionOptions::DISABLED => "Disabled",
                GuildProtectionOptions::NAME => "Name",
                GuildProtectionOptions::ICON => "Icon",
                _ => "Unknown",
            };

            flags.push(f);
        }

        write!(f, "{}", flags.join(", "))
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct FakeBotDetectionOptions: i32 {
        const DISABLED = 1 << 0;
        const BLOCK_ALL_BOTS = 1 << 1;
        const BLOCK_ALL_UNKNOWN_BOTS = 1 << 2; // An unknown bot is one that is not on the whitelist nor is registered on fake bot database with official ids
        const NORMALIZE_NAMES = 1 << 3;
        const EXACT_NAME_CHECK = 1 << 4;
        const SIMILAR_NAME_CHECK = 1 << 5;
    }
}

impl std::fmt::Display for FakeBotDetectionOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        for flag in self.iter() {
            let f = match flag {
                FakeBotDetectionOptions::DISABLED => "Disabled",
                FakeBotDetectionOptions::BLOCK_ALL_BOTS => "Block All Bots",
                FakeBotDetectionOptions::BLOCK_ALL_UNKNOWN_BOTS => "Block All Unknown Bots",
                FakeBotDetectionOptions::NORMALIZE_NAMES => "Normalize Names",
                FakeBotDetectionOptions::EXACT_NAME_CHECK => "Exact Name Check",
                FakeBotDetectionOptions::SIMILAR_NAME_CHECK => "Similar Name Check",
                _ => "Unknown",
            };

            flags.push(f);
        }

        write!(f, "{}", flags.join(", "))
    }
}

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub struct AutoResponseMemberJoinOptions: i32 {
        const DISABLED = 1 << 0;
        const KICK_NEW_MEMBERS = 1 << 1;
        const BAN_NEW_MEMBERS = 1 << 2; // An unknown bot is one that is not on the whitelist nor is registered on fake bot database with official ids
    }
}

impl AutoResponseMemberJoinOptions {
    pub fn order() -> Vec<Self> {
        vec![
            AutoResponseMemberJoinOptions::DISABLED,
            AutoResponseMemberJoinOptions::BAN_NEW_MEMBERS,
            AutoResponseMemberJoinOptions::KICK_NEW_MEMBERS,
        ]
    }
}

impl std::fmt::Display for AutoResponseMemberJoinOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = Vec::new();

        for flag in self.iter() {
            let f = match flag {
                AutoResponseMemberJoinOptions::DISABLED => "Disabled",
                AutoResponseMemberJoinOptions::KICK_NEW_MEMBERS => "Kick New Members",
                AutoResponseMemberJoinOptions::BAN_NEW_MEMBERS => "Ban New Members",
                _ => "Unknown",
            };

            flags.push(f);
        }

        write!(f, "{}", flags.join(", "))
    }
}
