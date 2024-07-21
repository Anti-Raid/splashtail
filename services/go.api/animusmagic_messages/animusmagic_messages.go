package animusmagic_messages

import (
	"github.com/anti-raid/splashtail/core/go.std/animusmagic"
	"github.com/anti-raid/splashtail/core/go.std/ext_types"
	"github.com/anti-raid/splashtail/core/go.std/silverpelt"
	orderedmap "github.com/wk8/go-ordered-map/v2"
)

/*
   pub struct AmCheckCommandOptionsFlags: u8 {
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

   /// Flags of type AmCheckCommandOptionsFlags
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

type AmCheckCommandOptionsFlag uint8

const (
	AmCheckCommandOptionsFlagIgnoreCache                 AmCheckCommandOptionsFlag = 1 << 0
	AmCheckCommandOptionsFlagCacheResult                 AmCheckCommandOptionsFlag = 1 << 1
	AmCheckCommandOptionsFlagIgnoreModuleDisabled        AmCheckCommandOptionsFlag = 1 << 2
	AmCheckCommandOptionsFlagIgnoreCommandDisabled       AmCheckCommandOptionsFlag = 1 << 3
	AmCheckCommandOptionsFlagSkipCustomResolvedFitChecks AmCheckCommandOptionsFlag = 1 << 4
)

type AmCheckCommandOptions struct {
	Flags                       AmCheckCommandOptionsFlag             `json:"flags"`
	CustomResolvedKittycatPerms *[]string                             `json:"custom_resolved_kittycat_perms,omitempty"`
	CustomCommandConfiguration  *silverpelt.GuildCommandConfiguration `json:"custom_command_configuration,omitempty"`
	CustomModuleConfiguration   *silverpelt.GuildModuleConfiguration  `json:"custom_module_configuration,omitempty"`
}

type ClusterModules = []silverpelt.CanonicalModule

type BotAnimusMessage struct {
	Modules     *struct{} `json:"Modules,omitempty"`
	GuildsExist *struct {
		Guilds []string `json:"guilds"`
	} `json:"GuildsExist,omitempty"`
	BaseGuildUserInfo *struct {
		GuildID string `json:"guild_id"`
		UserID  string `json:"user_id"`
	} `json:"BaseGuildUserInfo,omitempty"`
	CheckCommandPermission *struct {
		GuildID             string                `json:"guild_id"`
		UserID              string                `json:"user_id"`
		Command             string                `json:"command"`
		CheckCommandOptions AmCheckCommandOptions `json:"opts"`
	} `json:"CheckCommandPermission,omitempty"`
	ExecutePerModuleFunction *struct {
		Module  string         `json:"module"`
		Toggle  string         `json:"toggle"`
		Options map[string]any `json:"options,omitempty"`
	} `json:"ExecutePerModuleFunction,omitempty"`
	/// Executes an operation on a setting
	SettingsOperation *struct {
		Fields  orderedmap.OrderedMap[string, any] `json:"fields"`
		Op      silverpelt.CanonicalOperationType  `json:"op"`
		Module  string                             `json:"module"`
		Setting string                             `json:"setting"`
		GuildID string                             `json:"guild_id"`
		UserID  string                             `json:"user_id"`
	} `json:"SettingsOperation,omitempty"`
}

func (b BotAnimusMessage) Message() {}

func (b BotAnimusMessage) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetBot
}

type BotAnimusResponse struct {
	OK *struct {
		Message string `json:"message"`
	} `json:"OK,omitempty"`
	Modules *struct {
		Modules ClusterModules `json:"modules"`
	} `json:"Modules,omitempty"`

	GuildsExist *struct {
		GuildsExist []uint8 `json:"guilds_exist"`
	} `json:"GuildsExist,omitempty"`

	BaseGuildUserInfo *struct {
		OwnerID   string                                  `json:"owner_id"`
		Name      string                                  `json:"name"`
		Icon      *string                                 `json:"icon"`
		Roles     []ext_types.SerenityRole                `json:"roles"`
		UserRoles []string                                `json:"user_roles"`
		BotRoles  []string                                `json:"bot_roles"`
		Channels  []ext_types.GuildChannelWithPermissions `json:"channels"`
	} `json:"BaseGuildUserInfo,omitempty"`

	/// Returns the response of a command permission check
	CheckCommandPermission *struct {
		PermRes silverpelt.PermissionResult `json:"perm_res"`
		IsOk    bool                        `json:"is_ok"`
	} `json:"CheckCommandPermission,omitempty"`

	SettingsOperation *struct {
		Res silverpelt.CanonicalSettingsResult `json:"res"`
	} `json:"SettingsOperation,omitempty"`
}

func (b BotAnimusResponse) Response() {}

func (b BotAnimusResponse) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetBot
}

type JobserverMessage struct {
	// spawns a task and executes it if the execute argument is set.
	// If you already have both a task and a task create response, consider execute_task
	SpawnTask *struct {
		Name    string                 `json:"name"`
		Data    map[string]interface{} `json:"data"`
		Create  bool                   `json:"create"`
		Execute bool                   `json:"execute"`

		// If create is false, then task id must be set
		TaskID string `json:"task_id"`

		// The User ID who initiated the action
		UserID string `json:"user_id"`
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverMessage) Message() {}

func (b JobserverMessage) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetBot
}

type JobserverResponse struct {
	SpawnTask *struct {
		TaskID string `json:"task_id"`
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverResponse) Response() {}

func (b JobserverResponse) Target() animusmagic.AnimusTarget {
	return animusmagic.AnimusTargetJobserver
}
