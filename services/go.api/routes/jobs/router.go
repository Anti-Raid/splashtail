package jobs

import (
	"net/http"

	"go.api/api"
	"go.api/routes/jobs/endpoints/create_guild_job"
	"go.api/routes/jobs/endpoints/get_guild_job"
	"go.api/routes/jobs/endpoints/get_job_list"
	"go.api/routes/jobs/endpoints/ioauth_download_job"
	"go.std/splashcore"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Jobs"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to jobs"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern:      "/guilds/{guild_id}/jobs/{id}",
		OpId:         "get_guild_job",
		Method:       uapi.GET,
		Docs:         get_guild_job.Docs,
		Handler:      get_guild_job.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
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
		Pattern:      "/guilds/{guild_id}/jobs",
		OpId:         "get_job_list",
		Method:       uapi.GET,
		Docs:         get_job_list.Docs,
		Handler:      get_job_list.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
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
		Pattern: "/guilds/{guild_id}/jobs/{name}",
		OpId:    "create_guild_job",
		Method:  uapi.POST,
		Docs:    create_guild_job.Docs,
		Handler: create_guild_job.Route,
		Auth: []uapi.AuthType{
			{
				Type: splashcore.TargetTypeUser,
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
		Pattern: "/jobs/{id}/ioauth/download-link",
		OpId:    "ioauth_download_job",
		Method:  uapi.GET,
		Docs:    ioauth_download_job.Docs,
		Handler: ioauth_download_job.Route,
		ExtData: map[string]any{
			"ioauth": []string{"identify"},
		},
	}.Route(r)
}
