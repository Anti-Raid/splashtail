package backups

import "github.com/go-chi/chi/v5"

const tagName = "Backups"

type Router struct{}

func (r Router) Tag() (string, string) {
	return tagName, "Backup-related APIs"
}

func (m Router) Routes(r *chi.Mux) {

}
