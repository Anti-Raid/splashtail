package core

import (
	"github.com/anti-raid/splashtail/services/go.api/routes/core/endpoints/get_asset"
	"github.com/anti-raid/splashtail/services/go.api/routes/core/endpoints/get_cluster_modules"
	"github.com/anti-raid/splashtail/services/go.api/routes/core/endpoints/get_clusters_health"
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
		OpId:    "get_clusters_health",
		Method:  uapi.GET,
		Docs:    get_clusters_health.Docs,
		Handler: get_clusters_health.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/assets/{asset}",
		OpId:    "get_asset",
		Method:  uapi.GET,
		Docs:    get_asset.Docs,
		Handler: get_asset.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/clusters/{clusterId}/modules",
		OpId:    "get_cluster_modules",
		Method:  uapi.GET,
		Docs:    get_cluster_modules.Docs,
		Handler: get_cluster_modules.Route,
	}.Route(r)
}
