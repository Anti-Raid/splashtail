package auth

import (
	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/create_ioauth_login"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/create_oauth2_login"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/create_user_session"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/get_user_sessions"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/revoke_user_session"
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/test_auth"

	"github.com/go-chi/chi/v5"
	"github.com/infinitybotlist/eureka/uapi"
)

const tagName = "Auth"

type Router struct{}

func (r Router) Tag() (string, string) {
	return tagName, "Authentication APIs"
}

func (m Router) Routes(r *chi.Mux) {
	uapi.Route{
		Pattern: "/ioauth/login",
		OpId:    "create_ioauth_login",
		Method:  uapi.GET,
		Docs:    create_ioauth_login.Docs,
		Handler: create_ioauth_login.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/auth/test",
		OpId:    "test_auth",
		Method:  uapi.POST,
		Docs:    test_auth.Docs,
		Handler: test_auth.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/oauth2",
		OpId:    "create_oauth2_login",
		Method:  uapi.POST,
		Docs:    create_oauth2_login.Docs,
		Handler: create_oauth2_login.Route,
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/sessions",
		OpId:    "get_user_sessions",
		Method:  uapi.GET,
		Docs:    get_user_sessions.Docs,
		Handler: get_user_sessions.Route,
		Auth: []uapi.AuthType{
			{
				URLVar:       "user_id",
				Type:         types.TargetTypeUser,
				AllowedScope: "ban_exempt",
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/sessions",
		OpId:    "create_user_session",
		Method:  uapi.GET,
		Docs:    create_user_session.Docs,
		Handler: create_user_session.Route,
		Auth: []uapi.AuthType{
			{
				URLVar: "user_id",
				Type:   types.TargetTypeUser,
			},
		},
	}.Route(r)

	uapi.Route{
		Pattern: "/users/{user_id}/sessions/{session_id}",
		OpId:    "revoke_user_session",
		Method:  uapi.DELETE,
		Docs:    revoke_user_session.Docs,
		Handler: revoke_user_session.Route,
		Auth: []uapi.AuthType{
			{
				URLVar:       "user_id",
				Type:         types.TargetTypeUser,
				AllowedScope: "ban_exempt",
			},
		},
	}.Route(r)
}
