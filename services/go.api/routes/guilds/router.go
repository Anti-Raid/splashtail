package guilds

import (
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/api"
	"go.api/routes/guilds/endpoints/get_all_command_configurations"
	"go.api/routes/guilds/endpoints/get_module_configurations"
	"go.api/routes/guilds/endpoints/get_staff_team"
	"go.api/routes/guilds/endpoints/patch_command_configuration"
	"go.api/routes/guilds/endpoints/patch_module_configuration"
	"go.api/routes/guilds/endpoints/settings_execute"
	"go.std/splashcore"
)

const tagName = "Users"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to AntiRaid guilds"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/guilds/{guild_id}/staff-team",
		OpId:    "get_staff_team",
		Method:  uapi.GET,
		Docs:    get_staff_team.Docs,
		Handler: get_staff_team.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/guilds/{guild_id}/module-configurations",
		OpId:    "get_module_configurations",
		Method:  uapi.GET,
		Docs:    get_module_configurations.Docs,
		Handler: get_module_configurations.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
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
		Pattern: "/guilds/{guild_id}/module-configurations",
		OpId:    "patch_module_configuration",
		Method:  uapi.PATCH,
		Docs:    patch_module_configuration.Docs,
		Handler: patch_module_configuration.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil, // Authz is performed in the handler itself
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/guilds/{guild_id}/command-configurations",
		OpId:    "get_all_command_configurations",
		Method:  uapi.GET,
		Docs:    get_all_command_configurations.Docs,
		Handler: get_all_command_configurations.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
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
		Pattern: "/guilds/{guild_id}/command-configurations",
		OpId:    "patch_command_configuration",
		Method:  uapi.PATCH,
		Docs:    patch_command_configuration.Docs,
		Handler: patch_command_configuration.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil, // Authz is performed in the handler itself
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/guilds/{guild_id}/settings",
		OpId:    "settings_execute",
		Method:  uapi.POST,
		Docs:    settings_execute.Docs,
		Handler: settings_execute.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil, // Authz is performed in the handler itself
		},
	}.Route(r)
}
