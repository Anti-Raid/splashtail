package core

import (
	"github.com/anti-raid/splashtail/webserver/routes/core/endpoints/get_cluster_health"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Core"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to core functionality"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/clusters/health",
		OpId:    "get_cluster_health",
		Method:  uapi.GET,
		Docs:    get_cluster_health.Docs,
		Handler: get_cluster_health.Route,
	}.Route(r)
}
