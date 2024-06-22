package guilds

import (
	"net/http"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/routes/guilds/endpoints/get_all_command_configurations"
	"github.com/anti-raid/splashtail/webserver/routes/guilds/endpoints/get_command_configurations"
	"github.com/anti-raid/splashtail/webserver/routes/guilds/endpoints/get_module_configurations"
	"github.com/anti-raid/splashtail/webserver/routes/guilds/endpoints/toggle_module"
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
		OpId:    "get_module_configurations",
		Method:  uapi.GET,
		Docs:    get_module_configurations.Docs,
		Handler: get_module_configurations.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "modules list"
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/guilds/{guild_id}/command-configurations",
		OpId:    "get_all_command_configurations",
		Method:  uapi.GET,
		Docs:    get_all_command_configurations.Docs,
		Handler: get_all_command_configurations.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "commands list"
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/guilds/{guild_id}/commands/{command}/configurations",
		OpId:    "get_command_configurations",
		Method:  uapi.GET,
		Docs:    get_command_configurations.Docs,
		Handler: get_command_configurations.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "commands list"
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/guilds/{guild_id}/toggle-module",
		OpId:    "toggle_module",
		Method:  uapi.PUT,
		Docs:    toggle_module.Docs,
		Handler: toggle_module.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					if r.URL.Query().Get("disabled") == "true" {
						return "modules disable"
					}

					return "modules enable"
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)
}
