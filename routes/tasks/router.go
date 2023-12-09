package tasks

import (
	"splashtail/routes/tasks/endpoints/create_task"
	"splashtail/routes/tasks/endpoints/get_task"
	"splashtail/routes/tasks/endpoints/ioauth_download_task"
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

	uapi.Route{
		Pattern: "/entities/{id}/tasks/{name}",
		OpId:    "create_task",
		Method:  uapi.POST,
		Docs:    create_task.Docs,
		Handler: create_task.Route,
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

	uapi.Route{
		Pattern: "/tasks/{id}/ioauth/download-link",
		OpId:    "ioauth_download_task",
		Method:  uapi.GET,
		Docs:    ioauth_download_task.Docs,
		Handler: ioauth_download_task.Route,
		ExtData: map[string]any{
			"ioauth": []string{"identify", "guilds"},
		},
	}.Route(r)
}
