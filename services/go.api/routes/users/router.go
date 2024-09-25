package users

import (
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/api"
	"go.api/routes/users/endpoints/get_user"
	"go.api/routes/users/endpoints/get_user_guild_base_info"
	"go.api/routes/users/endpoints/get_user_guilds"
	"go.std/splashcore"
)

const tagName = "Users"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to AntiRaid users"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/users/{id}",
		OpId:    "get_user",
		Method:  uapi.GET,
		Docs:    get_user.Docs,
		Handler: get_user.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/users/@me/guilds",
		OpId:    "get_user_guilds",
		Method:  uapi.GET,
		Docs:    get_user_guilds.Docs,
		Handler: get_user_guilds.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil,
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/@me/guilds/{guild_id}",
		OpId:    "get_user_guild_base_info",
		Method:  uapi.GET,
		Docs:    get_user_guild_base_info.Docs,
		Handler: get_user_guild_base_info.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "" // No extra permissions are needed
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)
}
