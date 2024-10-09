package gitlogs

import (
	"time"

	"github.com/git-logs/client/webserver/config"
	"github.com/git-logs/client/webserver/mapofmu"
	"github.com/git-logs/client/webserver/ontos"
	"github.com/git-logs/client/webserver/state"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/infinitybotlist/eureka/zapchi"
	webserverstate "go.api/state"
)

// Sets up the git-logs bot
func Setup() *chi.Mux {
	state.Discord = webserverstate.Discord
	state.Pool = webserverstate.Pool
	state.MapMutex = mapofmu.New[string]()
	state.Logger = webserverstate.Logger
	state.Config = &config.Config{
		PostgresURL: webserverstate.Config.Meta.PostgresURL,
		APIUrl:      webserverstate.Config.Sites.API + "/integrations/gitlogs",
		GetTable: func(table string) string {
			if table == state.TableGuilds {
				return "guilds"
			}
			return "gitlogs__" + table
		},
	}

	// Prepare for embedding
	state.PrepareForEmbedding()

	state.ApplyMigrations()

	r := chi.NewMux()

	r.Use(zapchi.Logger(state.Logger.Sugar().Named("zapchi"), "git-logs"), middleware.Recoverer, middleware.RealIP, middleware.RequestID, middleware.Timeout(60*time.Second))

	// Webhook route
	r.Get("/kittycat", ontos.GetWebhookRoute)
	r.Post("/kittycat", ontos.HandleWebhookRoute)
	r.HandleFunc("/", ontos.IndexPage)
	r.HandleFunc("/audit", ontos.AuditEvent)

	// API
	r.HandleFunc("/api/counts", ontos.ApiStats)
	r.HandleFunc("/api/events/listview", ontos.ApiEventsListView)
	r.HandleFunc("/api/events/csview", ontos.ApiEventsCommaSepView)

	return r
}
