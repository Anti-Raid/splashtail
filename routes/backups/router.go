package backups

import (
	"splashtail/api"
	"splashtail/routes/backups/endpoints/create_backup"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Backups"

type Router struct{}

func (r Router) Tag() (string, string) {
	return tagName, "Backup-related APIs"
}

func (m Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/servers/{id}/backups",
		OpId:    "create_backup",
		Method:  uapi.POST,
		Docs:    create_backup.Docs,
		Handler: create_backup.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   api.TargetTypeServer,
			},
		},
	}.Route(r)
}
