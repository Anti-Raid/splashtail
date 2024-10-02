use serenity::all::User;

pub(crate) mod punishment_actions {
    use async_trait::async_trait;
    use serenity::all::{EditMember, Timestamp};
    use silverpelt::punishments::{
        CreatePunishmentAction, PunishmentAction, PunishmentActionData,
    };

    pub struct CreateTimeoutAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateTimeoutAction {
        fn name(&self) -> &'static str {
            "Timeout User"
        }

        fn syntax(&self) -> &'static str {
            "timeout"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<Box<dyn PunishmentAction>>, silverpelt::Error> {
            if s == "timeout" {
                Ok(Some(Box::new(TimeoutAction)))
            } else {
                Ok(None)
            }
        }
    }

    pub struct TimeoutAction;

    #[async_trait]
    impl PunishmentAction for TimeoutAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateTimeoutAction)
        }

        fn string_form(&self) -> String {
            "timeout".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String,
        ) -> Result<(), silverpelt::Error> {
            let timeout_duration = chrono::Duration::minutes(5);
            let new_time = chrono::Utc::now() + timeout_duration;

            bot_member
                .guild_id
                .edit_member(
                    &data.cache_http.http,
                    user_id,
                    EditMember::new()
                        .disable_communication_until(Timestamp::from(new_time))
                        .audit_log_reason(&reason),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String,
        ) -> Result<(), silverpelt::Error> {
            bot_member
                .guild_id
                .edit_member(
                    &data.cache_http.http,
                    user_id,
                    EditMember::new()
                        .enable_communication()
                        .audit_log_reason(&reason),
                )
                .await?;

            Ok(())
        }
    }

    pub struct CreateKickAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateKickAction {
        fn name(&self) -> &'static str {
            "Kick User"
        }

        fn syntax(&self) -> &'static str {
            "kick"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<Box<dyn PunishmentAction>>, silverpelt::Error> {
            if s == "kick" {
                Ok(Some(Box::new(KickAction)))
            } else {
                Ok(None)
            }
        }
    }

    pub struct KickAction;

    #[async_trait]
    impl PunishmentAction for KickAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateKickAction)
        }

        fn string_form(&self) -> String {
            "kick".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String,
        ) -> Result<(), silverpelt::Error> {
            bot_member
                .guild_id
                .kick(
                    &data.cache_http.http,
                    user_id,
                    Some(&reason),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            _data: &PunishmentActionData,
            _user_id: serenity::all::UserId,
            _bot_member: &mut serenity::all::Member,
            _reason: String,
        ) -> Result<(), silverpelt::Error> {
            Ok(()) // No-op
        }
    }

    pub struct CreateBanAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateBanAction {
        fn name(&self) -> &'static str {
            "Ban User"
        }

        fn syntax(&self) -> &'static str {
            "ban"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<Box<dyn PunishmentAction>>, silverpelt::Error> {
            if s == "ban" {
                Ok(Some(Box::new(BanAction)))
            } else {
                Ok(None)
            }
        }
    }

    pub struct BanAction;

    #[async_trait]
    impl PunishmentAction for BanAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateBanAction)
        }

        fn string_form(&self) -> String {
            "ban".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String
        ) -> Result<(), silverpelt::Error> {
            bot_member
                .guild_id
                .ban(
                    &data.cache_http.http,
                    user_id,
                    0,
                    Some(&reason),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String
        ) -> Result<(), silverpelt::Error> {
            bot_member
                .guild_id
                .unban(
                    &data.cache_http.http,
                    user_id,
                    Some(&reason),
                )
                .await?;

            Ok(())
        }
    }

    pub struct CreateRemoveAllRolesAction;

    #[async_trait]
    impl CreatePunishmentAction for CreateRemoveAllRolesAction {
        fn name(&self) -> &'static str {
            "Remove All Roles"
        }

        fn syntax(&self) -> &'static str {
            "remove_all_roles"
        }

        fn to_punishment_action(
            &self,
            s: &str,
        ) -> Result<Option<Box<dyn PunishmentAction>>, silverpelt::Error> {
            if s == "remove_all_roles" {
                Ok(Some(Box::new(RemoveAllRolesAction)))
            } else {
                Ok(None)
            }
        }
    }

    pub struct RemoveAllRolesAction;

    #[async_trait]
    impl PunishmentAction for RemoveAllRolesAction {
        fn creator(&self) -> Box<dyn CreatePunishmentAction> {
            Box::new(CreateRemoveAllRolesAction)
        }

        fn string_form(&self) -> String {
            "remove_all_roles".to_string()
        }

        async fn create(
            &self,
            data: &PunishmentActionData,
            user_id: serenity::all::UserId,
            bot_member: &mut serenity::all::Member,
            reason: String,
        ) -> Result<(), silverpelt::Error> {
            bot_member
                .guild_id
                .edit_member(
                    &data.cache_http.http,
                    user_id,
                    EditMember::new()
                        .roles(Vec::new())
                        .audit_log_reason(&reason),
                )
                .await?;

            Ok(())
        }

        async fn revert(
            &self,
            _data: &PunishmentActionData,
            _user_id: serenity::all::UserId,
            _bot_member: &mut serenity::all::Member,
            _reason: String,
        ) -> Result<(), silverpelt::Error> {
            Ok(()) // No-op
        }
    }
}

pub fn username(m: &User) -> String {
    if let Some(ref global_name) = m.global_name {
        global_name.to_string()
    } else {
        m.tag()
    }
}

pub fn to_log_format(moderator: &User, member: &User, reason: &str) -> String {
    format!(
        "{} | Handled '{}' for reason '{}'",
        username(moderator),
        username(member),
        reason
    )
}
