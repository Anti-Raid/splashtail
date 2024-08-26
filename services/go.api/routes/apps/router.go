package apps

import (
	"go.api/api"
	"go.api/routes/apps/endpoints/create_app"
	"go.api/routes/apps/endpoints/get_apps_list"
	"go.api/routes/apps/endpoints/get_apps_meta"
	"go.std/splashcore"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Apps"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to apps and interviews for positions on our list."
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/apps/meta",
		OpId:    "get_apps_meta",
		Method:  uapi.GET,
		Docs:    get_apps_meta.Docs,
		Handler: get_apps_meta.Route,
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil,
		},
	}.Route(r)
	uapi.Route{
		Pattern: "/apps",
		OpId:    "get_apps_list",
		Method:  uapi.GET,
		Docs:    get_apps_list.Docs,
		Handler: get_apps_list.Route,
		Auth: []uapi.AuthType{
			{
				Type:         splashcore.TargetTypeUser,
				AllowedScope: "ban_exempt", // Ensure banned users can view their own apps
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil,
		},
	}.Route(r)
	uapi.Route{
		Pattern: "/apps",
		OpId:    "create_app",
		Method:  uapi.POST,
		Docs:    create_app.Docs,
		Handler: create_app.Route,
		Auth: []uapi.AuthType{
			{
				Type:         splashcore.TargetTypeUser,
				AllowedScope: "ban_exempt", // Ensure banned users can create apps
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil,
		},
	}.Route(r)
}
