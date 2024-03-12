package guilds

import (
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/routes/guilds/endpoints/get_module_configuration"
	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Users"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to AntiRaid guilds"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/users/{user_id}/guilds/{guild_id}/module-configurations",
		OpId:    "get_user_guild_base_info",
		Method:  uapi.GET,
		Docs:    get_module_configuration.Docs,
		Handler: get_module_configuration.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
	}.Route(r)
}
