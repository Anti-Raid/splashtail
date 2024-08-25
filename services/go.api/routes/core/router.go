package core

import (
	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
	"go.api/routes/core/endpoints/get_api_config"
	"go.api/routes/core/endpoints/get_cluster_modules"
	"go.api/routes/core/endpoints/get_clusters_health"
)

const tagName = "Core"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to core functionality"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/config",
		OpId:    "get_api_config",
		Method:  uapi.GET,
		Docs:    get_api_config.Docs,
		Handler: get_api_config.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/clusters/health",
		OpId:    "get_clusters_health",
		Method:  uapi.GET,
		Docs:    get_clusters_health.Docs,
		Handler: get_clusters_health.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/clusters/{clusterId}/modules",
		OpId:    "get_cluster_modules",
		Method:  uapi.GET,
		Docs:    get_cluster_modules.Docs,
		Handler: get_cluster_modules.Route,
	}.Route(r)
}
