package auth

import (
	"github.com/anti-raid/splashtail/webserver/routes/auth/endpoints/create_ioauth_login"
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
}
