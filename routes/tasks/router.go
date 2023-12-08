package tasks

import (
	"splashtail/routes/tasks/endpoints/get_task"
	"splashtail/types"

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
		Pattern:      "/entities/{id}/tasks/{tid}",
		OpId:         "get_task",
		Method:       uapi.GET,
		Docs:         get_task.Docs,
		Handler:      get_task.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   types.TargetTypeServer,
			},
			{
				URLVar: "id",
				Type:   types.TargetTypeUser,
			},
		},
	}.Route(r)
}
