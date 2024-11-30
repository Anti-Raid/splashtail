package rpc_messages

import (
	"github.com/Anti-Raid/corelib_go/ext_types"
	"github.com/Anti-Raid/corelib_go/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

type BaseGuildUserInfo struct {
	OwnerID   string                                  `json:"owner_id"`
	Name      string                                  `json:"name"`
	Icon      *string                                 `json:"icon"`
	Roles     []ext_types.SerenityRole                `json:"roles"`
	UserRoles []string                                `json:"user_roles"`
	BotRoles  []string                                `json:"bot_roles"`
	Channels  []ext_types.GuildChannelWithPermissions `json:"channels"`
}

type CheckCommandPermission struct {
	PermRes silverpelt.PermissionResult `json:"perm_res"`
	IsOk    bool                        `json:"is_ok"`
}

type CheckCommandPermissionRequest struct {
	Command string                 `json:"command"`
	Opts    RpcCheckCommandOptions `json:"opts"`
}

type ClearModulesEnabledCacheRequest struct {
	GuildID string `json:"guild_id,omitempty"`
	Module  string `json:"module,omitempty"`
}

type ClearModulesEnabledCacheResponse struct{}

type SettingsOperationRequest struct {
	Fields  orderedmap.OrderedMap[string, any] `json:"fields"`
	Op      silverpelt.CanonicalOperationType  `json:"op"`
	Setting string                             `json:"setting"`
}

/*
   pub struct RpcCheckCommandOptionsFlags: u8 {
       /// Whether or not to ignore the cache
       const IGNORE_CACHE = 1 << 0;
       /// Whether or not to cache the result at all
       const CACHE_RESULT = 1 << 1;
       /// Whether or not to ignore the fact that the module is disabled in the guild
       const IGNORE_MODULE_DISABLED = 1 << 2;
       /// Whether or not to ignore the fact that the command is disabled in the guild
       const IGNORE_COMMAND_DISABLED = 1 << 3;
       /// Skip custom resolved kittycat permission fit 'checks' (AKA does the user have the actual permissions ofthe custom resolved permissions)
       const SKIP_CUSTOM_RESOLVED_FIT_CHECKS = 1 << 4;
   }

   /// Flags of type RpcCheckCommandOptionsFlags
   #[serde(default)]
   pub flags: u8,

   /// What custom resolved permissions to use for the user. Note that ensure_user_has_custom_resolved must be true to ensure that the user has all the permissions in the custom_resolved_kittycat_perms
   ///
   /// API needs this for limiting the permissions of a user, allows setting custom resolved perms
   #[serde(default)]
   pub custom_resolved_kittycat_perms: Option<Vec<String>>,

   /// Custom permission checks to use
   #[serde(default)]
   pub custom_command_configuration: Option<Box<silverpelt::GuildCommandConfiguration>>,

   /// Custom permission checks to use
   #[serde(default)]
   pub custom_module_configuration: Option<Box<silverpelt::GuildModuleConfiguration>>,

*/

type RpcCheckCommandOptionsFlag uint8

const (
	RpcCheckCommandOptionsFlagIgnoreModuleDisabled  RpcCheckCommandOptionsFlag = 1 << 2
	RpcCheckCommandOptionsFlagIgnoreCommandDisabled RpcCheckCommandOptionsFlag = 1 << 3
)

type RpcCheckCommandOptions struct {
	Flags                       RpcCheckCommandOptionsFlag            `json:"flags"`
	CustomResolvedKittycatPerms *[]string                             `json:"custom_resolved_kittycat_perms,omitempty"`
	CustomCommandConfiguration  *silverpelt.GuildCommandConfiguration `json:"custom_command_configuration,omitempty"`
	CustomModuleConfiguration   *silverpelt.GuildModuleConfiguration  `json:"custom_module_configuration,omitempty"`
	ChannelID                   string                                `json:"channel_id,omitempty"`
}

type CanonicalSettingsResult struct {
	Ok *struct {
		Fields []orderedmap.OrderedMap[string, any] `json:"fields"`
	} `json:"Ok"`
	Err *struct {
		Error silverpelt.CanonicalSettingsError `json:"error"`
	} `json:"Err"`
}

type JobserverSpawn struct {
	Name    string                 `json:"name"`
	Data    map[string]interface{} `json:"data"`
	Create  bool                   `json:"create"`
	Execute bool                   `json:"execute"`

	// If create is false, then `id`` must be set
	ID string `json:"id"`

	// The User ID who initiated the action
	UserID string `json:"user_id"`
}

type JobserverSpawnResponse struct {
	ID string `json:"id"`
}

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTemplateRequest {
    pub args: serde_json::Value,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecuteTemplateResponse {
    Ok { result: Option<serde_json::Value> },
    ExecErr { error: String },
    PermissionError { res: PermissionResult },
}
*/

type ExecuteTemplateRequest struct {
	Args     any    `json:"args"`
	Template string `json:"template"`
}

type ExecuteTemplateResponse struct {
	Ok *struct {
		Result any `json:"result"`
	} `json:"Ok,omitempty"`
	ExecErr *struct {
		Error string `json:"error"`
	} `json:"ExecErr,omitempty"`
	PermissionError *struct {
		Res silverpelt.PermissionResult `json:"res"`
	} `json:"PermissionError,omitempty"`
}
