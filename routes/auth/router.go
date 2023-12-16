package auth

import (
	"github.com/anti-raid/splashtail/routes/auth/endpoints/create_ioauth_login"

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
}
