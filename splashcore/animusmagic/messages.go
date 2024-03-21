// This package defines all possible responses to an action
package animusmagic

import (
	"github.com/anti-raid/splashtail/splashcore/silverpelt"
	"github.com/anti-raid/splashtail/splashcore/types"
)

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
		GuildID                     string   `json:"guild_id"`
		UserID                      string   `json:"user_id"`
		Command                     string   `json:"command"`
		CustomResolvedKittycatPerms []string `json:"custom_resolved_kittycat_perms,omitempty"`
		EnsureUserHasCustomResolved *bool    `json:"ensure_user_has_custom_resolved,omitempty"`
	} `json:"CheckCommandPermission,omitempty"`
	ToggleModule *struct {
		GuildID string `json:"guild_id"`
		Module  string `json:"module"`
		Enabled bool   `json:"enabled"`
	} `json:"ToggleModule,omitempty"`
}

func (b BotAnimusMessage) Message() {}

func (b BotAnimusMessage) Target() AnimusTarget {
	return AnimusTargetBot
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
	}

	BaseGuildUserInfo *types.UserGuildBaseData

	/// Returns the response of a command permission check
	CheckCommandPermission *struct {
		PermRes silverpelt.PermissionResult `json:"perm_res"`
		IsOk    bool                        `json:"is_ok"`
	}
}

func (b BotAnimusResponse) Response() {}

func (b BotAnimusResponse) Target() AnimusTarget {
	return AnimusTargetBot
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
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverMessage) Message() {}

func (b JobserverMessage) Target() AnimusTarget {
	return AnimusTargetBot
}

type JobserverResponse struct {
	SpawnTask *struct {
		TaskID string `json:"task_id"`
	} `json:"SpawnTask,omitempty"`
}

func (b JobserverResponse) Response() {}

func (b JobserverResponse) Target() AnimusTarget {
	return AnimusTargetBot
}
