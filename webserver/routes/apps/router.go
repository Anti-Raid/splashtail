package apps

import (
	"github.com/anti-raid/splashtail/splashcore/types"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/routes/apps/endpoints/create_app"
	"github.com/anti-raid/splashtail/webserver/routes/apps/endpoints/get_apps_list"
	"github.com/anti-raid/splashtail/webserver/routes/apps/endpoints/get_apps_meta"

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
			api.PERMISSION_CHECK_KEY: nil, // The permission check is handled in the actual function
		},
	}.Route(r)
	uapi.Route{
		Pattern: "/users/{user_id}/apps",
		OpId:    "get_apps_list",
		Method:  uapi.GET,
		Docs:    get_apps_list.Docs,
		Handler: get_apps_list.Route,
		Auth: []uapi.AuthType{
			{
				URLVar:       "user_id",
				Type:         types.TargetTypeUser,
				AllowedScope: "ban_exempt", // Ensure banned users can view their own apps
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil, // The permission check is handled in the actual function
		},
	}.Route(r)
	uapi.Route{
		Pattern: "/users/{user_id}/apps",
		OpId:    "create_app",
		Method:  uapi.POST,
		Docs:    create_app.Docs,
		Handler: create_app.Route,
		Auth: []uapi.AuthType{
			{
				URLVar:       "user_id",
				Type:         types.TargetTypeUser,
				AllowedScope: "ban_exempt", // Ensure banned users can create apps
			},
		},
		ExtData: map[string]any{
			api.PERMISSION_CHECK_KEY: nil, // The permission check is handled in the actual function
		},
	}.Route(r)
}
