package tasks

import (
	api "splashtail/api"
	"splashtail/routes/tasks/endpoints/get_task"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Tasks"

type Router struct{}

func (b Router) Tag() (string, string) {
	return tagName, "These API endpoints are related to tasks on IBL"
}

func (b Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern:      "/users/{id}/tasks/{tid}",
		OpId:         "get_task",
		Method:       uapi.GET,
		Docs:         get_task.Docs,
		Handler:      get_task.Route,
		AuthOptional: true,
		Auth: []uapi.AuthType{
			{
				URLVar: "id",
				Type:   api.TargetTypeUser,
			},
		},
	}.Route(r)
}
