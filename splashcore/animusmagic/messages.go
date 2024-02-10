package animusmagic

import (
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/splashcore/types/silverpelt"
)

type ClusterModules = []silverpelt.CanonicalModule

type CommonAnimusMessage struct {
	Probe        *struct{}    `json:"Probe,omitempty"`
	AnimusTarget AnimusTarget `json:"-"`
}

func (b CommonAnimusMessage) Message() {}

func (b CommonAnimusMessage) Target() AnimusTarget {
	return b.AnimusTarget
}

type CommonAnimusResponse struct {
	Probe *struct {
		Message string `json:"message"`
	} `json:"Probe,omitempty"`
	AnimusTarget AnimusTarget `json:"-"`
}

func (b CommonAnimusResponse) Response() {}

func (b CommonAnimusResponse) Target() AnimusTarget {
	return b.AnimusTarget
}

type BotAnimusMessage struct {
	Modules     *struct{} `json:"Modules,omitempty"`
	GuildsExist *struct {
		Guilds []string `json:"guilds"`
	} `json:"GuildsExist,omitempty"`
	BaseGuildUserInfo *struct {
		GuildID string `json:"guild_id"`
		UserID  string `json:"user_id"`
	} `json:"BaseGuildUserInfo,omitempty"`
}

func (b BotAnimusMessage) Message() {}

func (b BotAnimusMessage) Target() AnimusTarget {
	return AnimusTargetBot
}

type BotAnimusResponse struct {
	Modules *struct {
		Modules ClusterModules `json:"modules"`
	} `json:"Modules,omitempty"`

	GuildsExist *struct {
		GuildsExist []uint8 `json:"guilds_exist"`
	}

	BaseGuildUserInfo *types.UserGuildBaseData
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
