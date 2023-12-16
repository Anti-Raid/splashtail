package platform

import (
	"github.com/anti-raid/splashtail/routes/platform/endpoints/clear_user"
	"github.com/anti-raid/splashtail/routes/platform/endpoints/get_user"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Platform-Specific"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to platform specific endpoints such as fetching discord users etc"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/platform/user/{id}",
		OpId:    "get_user",
		Method:  uapi.GET,
		Docs:    get_user.Docs,
		Handler: get_user.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/platform/user/{id}",
		OpId:    "clear_user",
		Method:  uapi.DELETE,
		Docs:    clear_user.Docs,
		Handler: clear_user.Route,
	}.Route(r)
}
