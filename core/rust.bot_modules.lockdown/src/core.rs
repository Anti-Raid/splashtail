use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Change operation, commonly used in lockdown modes
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq)]
pub enum ChangeOp {
    Add,
    Remove,
}

impl std::fmt::Display for ChangeOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeOp::Add => write!(f, "Add"),
            ChangeOp::Remove => write!(f, "Remove"),
        }
    }
}

/// Returns the critical roles given a [PartialGuild](`serenity::all::PartialGuild`) and a set of member roles
pub fn get_critical_roles(
    pg: &serenity::all::PartialGuild,
    member_roles: &HashSet<serenity::all::RoleId>,
) -> Result<HashSet<serenity::all::RoleId>, silverpelt::Error> {
    if member_roles.is_empty() {
        // Find the everyone role
        let everyone_role = pg
            .roles
            .iter()
            .find(|r| r.id.get() == pg.id.get())
            .ok_or_else(|| silverpelt::Error::from("No @everyone role found"))?;

        Ok(std::iter::once(everyone_role.id).collect())
    } else {
        Ok(member_roles.clone())
    }
}

pub struct LockdownData {
    pub cache_http: botox::cache::CacheHttpImpl,
    pub pool: sqlx::PgPool,
    pub reqwest: reqwest::Client,
    pub object_store: Arc<splashcore_rs::objectstore::ObjectStore>,
    pub permodule_executor: Box<dyn splashcore_rs::permodule_functions::PermoduleFunctionExecutor>,
}

pub trait LockdownTestResult
where
    Self: Send + Sync,
{
    /// Returns whether the lockdown can be applied perfectly with the current server layout
    fn can_apply_perfectly(&self) -> bool;

    /// Returns a string representation of the result
    fn display_result(&self, pg: &serenity::all::PartialGuild) -> String;
}

/// To ensure two lockdowns don't conflict with each other, we need some information about what all lockdowns are handling
pub struct LockdownModeHandles {
    pub roles: HashSet<serenity::all::RoleId>,
    pub channels: HashSet<serenity::all::ChannelId>,
}

impl LockdownModeHandles {
    pub fn merge(&mut self, other: &LockdownModeHandles) {
        self.roles.extend(other.roles.iter().cloned());
        self.channels.extend(other.channels.iter().cloned());
    }
}

#[async_trait]
pub trait LockdownMode
where
    Self: Send + Sync,
{
    /// All lockdowns will be sorted by this value, with the highest value being the most specific and hence viewed first
    fn specificity(&self) -> i32;

    async fn test(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error>;

    /// Sets up the lockdown mode, returning any data to be stored in database
    async fn setup(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<serde_json::Value, silverpelt::Error>;

    async fn create(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error>;

    async fn revert(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error>;

    async fn handles(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
    ) -> Result<LockdownModeHandles, silverpelt::Error>;
}

/// Enum containing all variants
pub enum LockdownModes {
    QuickServerLockdown(qsl::QuickServerLockdown),
    TraditionalServerLockdown(tsl::TraditionalServerLockdown),
    SingleChannelLockdown(scl::SingleChannelLockdown),
    Unknown(Box<dyn LockdownMode>),
}

impl LockdownModes {
    pub fn from_string(s: &str) -> Result<Option<LockdownModes>, silverpelt::Error> {
        if s == "qsl" {
            Ok(Some(LockdownModes::QuickServerLockdown(
                qsl::QuickServerLockdown,
            )))
        } else if s == "tsl" {
            Ok(Some(LockdownModes::TraditionalServerLockdown(
                tsl::TraditionalServerLockdown,
            )))
        } else if s.starts_with("scl/") {
            let channel_id = s
                .strip_prefix("scl/")
                .ok_or_else(|| silverpelt::Error::from("Invalid SCL string"))?
                .parse()
                .map_err(|e| format!("Error while parsing channel ID: {}", e))?;
            Ok(Some(LockdownModes::SingleChannelLockdown(
                scl::SingleChannelLockdown(channel_id),
            )))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl LockdownMode for LockdownModes {
    fn specificity(&self) -> i32 {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => qsl.specificity(),
            LockdownModes::TraditionalServerLockdown(tsl) => tsl.specificity(),
            LockdownModes::SingleChannelLockdown(scl) => scl.specificity(),
            LockdownModes::Unknown(m) => m.specificity(),
        }
    }

    async fn test(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.test(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.test(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.test(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::Unknown(m) => m.test(lockdown_data, pg, critical_roles).await,
        }
    }

    async fn setup(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
    ) -> Result<serde_json::Value, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.setup(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.setup(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.setup(lockdown_data, pg, critical_roles).await
            }
            LockdownModes::Unknown(m) => m.setup(lockdown_data, pg, critical_roles).await,
        }
    }

    async fn create(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.create(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.create(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.create(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::Unknown(m) => {
                m.create(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
        }
    }

    async fn revert(
        &self,
        lockdown_data: &LockdownData,
        pg: &mut serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
        all_handles: &LockdownModeHandles,
    ) -> Result<(), silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.revert(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.revert(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.revert(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
            LockdownModes::Unknown(m) => {
                m.revert(lockdown_data, pg, critical_roles, data, all_handles)
                    .await
            }
        }
    }

    async fn handles(
        &self,
        lockdown_data: &LockdownData,
        pg: &serenity::all::PartialGuild,
        critical_roles: &HashSet<serenity::all::RoleId>,
        data: &serde_json::Value,
    ) -> Result<LockdownModeHandles, silverpelt::Error> {
        match self {
            LockdownModes::QuickServerLockdown(qsl) => {
                qsl.handles(lockdown_data, pg, critical_roles, data).await
            }
            LockdownModes::TraditionalServerLockdown(tsl) => {
                tsl.handles(lockdown_data, pg, critical_roles, data).await
            }
            LockdownModes::SingleChannelLockdown(scl) => {
                scl.handles(lockdown_data, pg, critical_roles, data).await
            }
            LockdownModes::Unknown(m) => m.handles(lockdown_data, pg, critical_roles, data).await,
        }
    }
}

/// Represents a lockdown
pub struct Lockdown {
    pub r#type: LockdownModes,
    pub data: serde_json::Value,
}

/// Represents a list of lockdowns
pub struct LockdownSet {
    pub lockdowns: Vec<Lockdown>,
}

impl LockdownSet {
    pub async fn guild<'a, E>(
        guild_id: serenity::all::GuildId,
        db: E,
    ) -> Result<Self, silverpelt::Error>
    where
        E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    {
        let data = sqlx::query!(
            "SELECT type, data FROM lockdown__guild_lockdowns WHERE guild_id = $1",
            guild_id.to_string(),
        )
        .fetch_all(db)
        .await?;

        let mut lockdowns = Vec::new();

        for row in data {
            let r#type = row.r#type;
            let data = row.data;

            let lockdown = match LockdownModes::from_string(&r#type) {
                Ok(Some(m)) => Lockdown { r#type: m, data },
                Ok(None) => continue,
                Err(e) => {
                    return Err(silverpelt::Error::from(format!(
                        "Error while parsing lockdown type: {}",
                        e
                    )))
                }
            };

            lockdowns.push(lockdown);
        }

        Ok(LockdownSet { lockdowns })
    }
}

/// Quick server lockdown
pub mod qsl {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// The base permissions for quick lockdown
    ///
    /// If any of these permissions are provided, quick lockdown cannot proceed
    static BASE_PERMS: [serenity::all::Permissions; 2] = [
        serenity::all::Permissions::VIEW_CHANNEL,
        serenity::all::Permissions::SEND_MESSAGES,
    ];

    static LOCKDOWN_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| serenity::all::Permissions::VIEW_CHANNEL);

    /// The result of a `test_quick_lockdown` call
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct QuickLockdownTestResult {
        /// Which roles need to be changed/fixed combined with the target perms
        pub changes_needed: std::collections::HashMap<
            serenity::all::RoleId,
            (ChangeOp, serenity::all::Permissions),
        >,
        /// The critical roles (either member roles or the `@everyone` role)
        pub critical_roles: HashSet<serenity::all::RoleId>,
    }

    impl LockdownTestResult for QuickLockdownTestResult {
        /// Returns whether the guild is in a state where quick lockdown can be applied perfectly
        fn can_apply_perfectly(&self) -> bool {
            self.changes_needed.is_empty()
        }

        fn display_result(&self, pg: &serenity::all::PartialGuild) -> String {
            let mut needed_changes = String::new();

            needed_changes.push_str("The following roles need to be changed:\n");
            for (role_id, perms) in self.changes_needed.iter() {
                let role_name = pg
                    .roles
                    .get(role_id)
                    .map(|r| r.name.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                needed_changes.push_str(&format!("Role: {} ({})\n", role_name, role_id));
                needed_changes.push_str(&format!("Permissions: {} {}\n", perms.0, perms.1));
                needed_changes.push('\n');
            }

            needed_changes
        }
    }

    pub struct QuickServerLockdown;

    impl QuickServerLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<
            std::collections::HashMap<serenity::all::RoleId, serenity::all::Permissions>,
            silverpelt::Error,
        > {
            let v: std::collections::HashMap<serenity::all::RoleId, serenity::all::Permissions> =
                serde_json::from_value(data.clone())
                    .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for QuickServerLockdown {
        // Lowest specificity
        fn specificity(&self) -> i32 {
            0
        }

        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            let mut changes_needed = std::collections::HashMap::new();

            // From here on out, we only need to care about critical and non critical roles
            for role in pg.roles.iter() {
                if critical_roles.contains(&role.id) {
                    let mut needed_perms = serenity::all::Permissions::empty();

                    let mut missing = false;
                    for perm in BASE_PERMS {
                        if !role.permissions.contains(perm) {
                            needed_perms |= perm;
                            missing = true;
                        }
                    }

                    if missing {
                        changes_needed.insert(role.id, (ChangeOp::Add, needed_perms));
                    }
                } else {
                    let mut perms_to_remove = serenity::all::Permissions::empty();

                    let mut needs_perms_removed = false;
                    for perm in BASE_PERMS {
                        if role.permissions.contains(perm) {
                            perms_to_remove |= perm;
                            needs_perms_removed = true;
                        }
                    }

                    if needs_perms_removed {
                        changes_needed.insert(role.id, (ChangeOp::Remove, perms_to_remove));
                    }
                }
            }

            Ok(Box::new(QuickLockdownTestResult {
                changes_needed,
                critical_roles: critical_roles.clone(),
            }))
        }

        async fn setup(
            &self,
            _lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            let mut map = serde_json::Map::new();

            for role in pg.roles.iter() {
                map.insert(
                    role.id.to_string(),
                    serde_json::Value::String(role.permissions.bits().to_string()),
                );
            }

            Ok(serde_json::Value::Object(map))
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let mut new_roles = Vec::new();
            for role in pg.roles.iter() {
                // If critical, lock it down
                if critical_roles.contains(&role.id) {
                    new_roles.push(
                        pg.id
                            .edit_role(
                                &lockdown_data.cache_http.http,
                                role.id,
                                serenity::all::EditRole::new().permissions(*LOCKDOWN_PERMS),
                            )
                            .await?,
                    );
                }
            }

            for role in new_roles {
                pg.roles.insert(role);
            }

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let old_permissions = Self::from_data(data)?;

            let mut new_roles = Vec::new();
            for role in pg.roles.iter() {
                if critical_roles.contains(&role.id) {
                    let perms = old_permissions.get(&role.id).copied().unwrap_or(
                        BASE_PERMS
                            .iter()
                            .copied()
                            .fold(serenity::all::Permissions::empty(), |acc, perm| acc | perm),
                    );

                    new_roles.push(
                        pg.id
                            .edit_role(
                                &lockdown_data.cache_http.http,
                                role.id,
                                serenity::all::EditRole::new().permissions(perms),
                            )
                            .await?,
                    );
                }
            }

            for role in new_roles {
                pg.roles.insert(role);
            }

            Ok(())
        }

        async fn handles(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandles, silverpelt::Error> {
            // QSL locks the critical roles
            Ok(LockdownModeHandles {
                roles: critical_roles.clone(),
                channels: HashSet::new(),
            })
        }
    }
}

/// Traditional server lockdown (lock all channels)
pub mod tsl {
    use super::*;
    use serde::{Deserialize, Serialize};

    static DENY_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| {
            serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::SEND_MESSAGES_IN_THREADS
                | serenity::all::Permissions::SEND_TTS_MESSAGES
                | serenity::all::Permissions::CONNECT
        });

    // The big advantage of TSL is the lack of constraints/tests regarding server layout
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct TraditionalLockdownTestResult;

    impl LockdownTestResult for TraditionalLockdownTestResult {
        fn can_apply_perfectly(&self) -> bool {
            true
        }

        fn display_result(&self, _pg: &serenity::all::PartialGuild) -> String {
            "".to_string()
        }
    }
    pub struct TraditionalServerLockdown;

    impl TraditionalServerLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<
            std::collections::HashMap<
                serenity::all::ChannelId,
                Vec<serenity::all::PermissionOverwrite>,
            >,
            silverpelt::Error,
        > {
            let v: std::collections::HashMap<
                serenity::all::ChannelId,
                Vec<serenity::all::PermissionOverwrite>,
            > = serde_json::from_value(data.clone())
                .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for TraditionalServerLockdown {
        // TSL > QSL as it updates all channels in a server
        fn specificity(&self) -> i32 {
            1
        }

        // TSL doesn't need to test anything so just return the result
        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            Ok(Box::new(TraditionalLockdownTestResult))
        }

        async fn setup(
            &self,
            lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            let channels = proxy_support::guild_channels(
                &lockdown_data.cache_http,
                &lockdown_data.reqwest,
                pg.id,
            )
            .await?;

            let mut map = serde_json::Map::new();

            for channel in channels.iter() {
                map.insert(
                    channel.id.to_string(),
                    serde_json::to_value(channel.permission_overwrites.clone())?,
                );
            }

            Ok(serde_json::Value::Object(map))
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let mut channels = proxy_support::guild_channels(
                &lockdown_data.cache_http,
                &lockdown_data.reqwest,
                pg.id,
            )
            .await?;

            for channel in channels.iter_mut() {
                if all_handles.channels.contains(&channel.id) {
                    continue; // Someone else is handling this channel
                }

                let mut overwrites = channel.permission_overwrites.to_vec();

                let mut nyset_overwrite = critical_roles.clone();
                for overwrite in overwrites.iter_mut() {
                    match overwrite.kind {
                        serenity::all::PermissionOverwriteType::Role(role_id) => {
                            if critical_roles.contains(&role_id) {
                                overwrite.deny = *DENY_PERMS;
                                nyset_overwrite.remove(&role_id);
                            }
                        }
                        _ => continue,
                    }
                }

                if !nyset_overwrite.is_empty() {
                    for critical_role in nyset_overwrite.iter() {
                        let overwrite = serenity::all::PermissionOverwrite {
                            allow: serenity::all::Permissions::empty(),
                            deny: *DENY_PERMS,
                            kind: serenity::all::PermissionOverwriteType::Role(*critical_role),
                        };

                        overwrites.push(overwrite);
                    }
                }

                channel
                    .edit(
                        &lockdown_data.cache_http.http,
                        serenity::all::EditChannel::new().permissions(overwrites),
                    )
                    .await?;
            }

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            pg: &mut serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let old_permissions = Self::from_data(data)?;

            let mut channels = proxy_support::guild_channels(
                &lockdown_data.cache_http,
                &lockdown_data.reqwest,
                pg.id,
            )
            .await?;

            for channel in channels.iter_mut() {
                if all_handles.channels.contains(&channel.id) {
                    continue; // Someone else is handling this channel
                }

                // TODO: Handle this slightly better (maybe only apply the changes to critical roles somehow)
                let Some(overwrites) = old_permissions.get(&channel.id).cloned() else {
                    continue;
                };

                channel
                    .edit(
                        &lockdown_data.cache_http.http,
                        serenity::all::EditChannel::new().permissions(overwrites),
                    )
                    .await?;
            }

            Ok(())
        }

        async fn handles(
            &self,
            lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandles, silverpelt::Error> {
            // TSL locks all channels, but *NOT* roles
            let channels = proxy_support::guild_channels(
                &lockdown_data.cache_http,
                &lockdown_data.reqwest,
                pg.id,
            )
            .await?;

            Ok(LockdownModeHandles {
                roles: HashSet::new(),
                channels: channels.iter().map(|c| c.id).collect(),
            })
        }
    }
}

/// Single channel lock
pub mod scl {
    use super::*;
    use serde::{Deserialize, Serialize};

    static DENY_PERMS: std::sync::LazyLock<serenity::all::Permissions> =
        std::sync::LazyLock::new(|| {
            serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::SEND_MESSAGES_IN_THREADS
                | serenity::all::Permissions::SEND_TTS_MESSAGES
                | serenity::all::Permissions::CONNECT
        });

    // The big advantage of TSL is the lack of constraints/tests regarding server layout
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct SingleChannelLockdownTestResult;

    impl LockdownTestResult for SingleChannelLockdownTestResult {
        fn can_apply_perfectly(&self) -> bool {
            true
        }

        fn display_result(&self, _pg: &serenity::all::PartialGuild) -> String {
            "".to_string()
        }
    }
    pub struct SingleChannelLockdown(pub serenity::all::ChannelId);

    impl SingleChannelLockdown {
        pub fn from_data(
            data: &serde_json::Value,
        ) -> Result<Vec<serenity::all::PermissionOverwrite>, silverpelt::Error> {
            let v: Vec<serenity::all::PermissionOverwrite> =
                serde_json::from_value(data.clone())
                    .map_err(|e| format!("Error while deserializing permissions: {}", e))?;

            Ok(v)
        }
    }

    #[async_trait]
    impl LockdownMode for SingleChannelLockdown {
        // SCL > TSL as it updates a single channel
        fn specificity(&self) -> i32 {
            2
        }

        // SCL doesn't need to test anything so just return the result
        async fn test(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<Box<dyn LockdownTestResult>, silverpelt::Error> {
            Ok(Box::new(SingleChannelLockdownTestResult))
        }

        async fn setup(
            &self,
            lockdown_data: &LockdownData,
            pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
        ) -> Result<serde_json::Value, silverpelt::Error> {
            let channels = proxy_support::guild_channels(
                &lockdown_data.cache_http,
                &lockdown_data.reqwest,
                pg.id,
            )
            .await?;

            let channel = channels
                .iter()
                .find(|c| c.id == self.0)
                .ok_or_else(|| silverpelt::Error::from("Channel not found"))?;

            Ok(serde_json::to_value(channel.permission_overwrites.clone())?)
        }

        async fn create(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let mut overwrites = Self::from_data(data)?;

            let mut nyset_overwrite = critical_roles.clone();
            for overwrite in overwrites.iter_mut() {
                match overwrite.kind {
                    serenity::all::PermissionOverwriteType::Role(role_id) => {
                        if critical_roles.contains(&role_id) {
                            overwrite.deny = *DENY_PERMS;
                            nyset_overwrite.remove(&role_id);
                        }
                    }
                    _ => continue,
                }
            }

            if !nyset_overwrite.is_empty() {
                for critical_role in nyset_overwrite.iter() {
                    let overwrite = serenity::all::PermissionOverwrite {
                        allow: serenity::all::Permissions::empty(),
                        deny: *DENY_PERMS,
                        kind: serenity::all::PermissionOverwriteType::Role(*critical_role),
                    };

                    overwrites.push(overwrite);
                }
            }

            self.0
                .edit(
                    &lockdown_data.cache_http.http,
                    serenity::all::EditChannel::new().permissions(overwrites),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            lockdown_data: &LockdownData,
            _pg: &mut serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
            data: &serde_json::Value,
            _all_handles: &LockdownModeHandles,
        ) -> Result<(), silverpelt::Error> {
            let overwrites = Self::from_data(data)?;

            self.0
                .edit(
                    &lockdown_data.cache_http.http,
                    serenity::all::EditChannel::new().permissions(overwrites),
                )
                .await?;

            Ok(())
        }

        async fn handles(
            &self,
            _lockdown_data: &LockdownData,
            _pg: &serenity::all::PartialGuild,
            _critical_roles: &HashSet<serenity::all::RoleId>,
            _data: &serde_json::Value,
        ) -> Result<LockdownModeHandles, silverpelt::Error> {
            // SCL locks a single channel
            Ok(LockdownModeHandles {
                roles: HashSet::new(),
                channels: std::iter::once(self.0).collect(),
            })
        }
    }
}
