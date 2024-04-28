package tasks

import (
	"net/http"

	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/routes/tasks/endpoints/create_guild_task"
	"github.com/anti-raid/splashtail/webserver/routes/tasks/endpoints/get_guild_task"
	"github.com/anti-raid/splashtail/webserver/routes/tasks/endpoints/get_task_list"
	"github.com/anti-raid/splashtail/webserver/routes/tasks/endpoints/ioauth_download_task"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Tasks"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to tasks"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern:      "/users/{id}/guilds/{guild_id}/tasks/{tid}",
		OpId:         "get_guild_task",
		Method:       uapi.GET,
		Docs:         get_guild_task.Docs,
		Handler:      get_guild_task.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "" // The command permission check happens inside the endpoint
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern:      "/users/{id}/guilds/{guild_id}/tasks",
		OpId:         "get_task_list",
		Method:       uapi.GET,
		Docs:         get_task_list.Docs,
		Handler:      get_task_list.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "" // The command permission check happens inside the endpoint
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{id}/guilds/{guild_id}/tasks/{name}",
		OpId:    "create_task",
		Method:  uapi.POST,
		Docs:    create_guild_task.Docs,
		Handler: create_guild_task.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   types.TargetTypeUser,
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: api.PermissionCheck{
				Command: func(d uapi.Route, r *http.Request) string {
					return "" // The command permission check happens inside the endpoint
				},
				GuildID: func(d uapi.Route, r *http.Request) string {
					return chi.URLParam(r, "guild_id")
				},
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/tasks/{id}/ioauth/download-link",
		OpId:    "ioauth_download_task",
		Method:  uapi.GET,
		Docs:    ioauth_download_task.Docs,
		Handler: ioauth_download_task.Route,
		ExtData: map[string]any{
			"ioauth": []string{"identify"},
		},
	}.Route(r)
}
