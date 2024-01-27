package webserver

import (
	"html/template"
	"net/http"
	"strings"
	"time"

	_ "embed"

	"github.com/anti-raid/splashtail/types"
	"github.com/anti-raid/splashtail/webserver/api"
	"github.com/anti-raid/splashtail/webserver/constants"
	"github.com/anti-raid/splashtail/webserver/routes/auth"
	"github.com/anti-raid/splashtail/webserver/routes/core"
	"github.com/anti-raid/splashtail/webserver/routes/platform"
	"github.com/anti-raid/splashtail/webserver/routes/tasks"
	"github.com/anti-raid/splashtail/webserver/routes/users"
	"github.com/anti-raid/splashtail/webserver/state"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/infinitybotlist/eureka/zapchi"
	jsoniter "github.com/json-iterator/go"
)

var json = jsoniter.ConfigCompatibleWithStandardLibrary

//go:embed docs/docs.html
var docsHTML string

var openapi []byte

// Simple middleware to handle CORS
func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// limit body to 10mb
		r.Body = http.MaxBytesReader(w, r.Body, 10*1024*1024)

		if r.Header.Get("User-Auth") != "" {
			if strings.HasPrefix(r.Header.Get("User-Auth"), "User ") {
				r.Header.Set("Authorization", r.Header.Get("User-Auth"))
			} else {
				r.Header.Set("Authorization", "User "+r.Header.Get("User-Auth"))
			}
		}

		if r.Header.Get("Server-Auth") != "" {
			if strings.HasPrefix(r.Header.Get("Server-Auth"), "Server ") {
				r.Header.Set("Authorization", r.Header.Get("Server-Auth"))
			} else {
				r.Header.Set("Authorization", "Server "+r.Header.Get("Server-Auth"))
			}
		}

		w.Header().Set("Access-Control-Allow-Origin", r.Header.Get("Origin"))
		w.Header().Set("Access-Control-Allow-Credentials", "true")
		w.Header().Set("Access-Control-Allow-Headers", "X-Client, Content-Type, Authorization, User-Auth, Server-Auth")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE")

		if r.Method == "OPTIONS" {
			w.Write([]byte{})
			return
		}

		w.Header().Set("Content-Type", "application/json")

		next.ServeHTTP(w, r)
	})
}

func CreateWebserver() *chi.Mux {
	docs.DocsSetupData = &docs.SetupData{
		URL:         state.Config.Sites.API.Parse(),
		ErrorStruct: types.ApiError{},
		Info: docs.Info{
			Title:          "Antiraid API",
			TermsOfService: "https://antiraid.xyz/terms",
			Version:        "7.0",
			Description:    "",
			Contact: docs.Contact{
				Name:  "Anti Raid Development",
				URL:   "https://antiraid.xyz",
				Email: "support@antiraid.gxyz",
			},
			License: docs.License{
				Name: "AGPL3",
				URL:  "https://opensource.org/licenses/AGPL3",
			},
		},
	}

	docs.Setup()

	docs.AddSecuritySchema("User", "User-Auth", "Requires a user token. Should be prefixed with `User ` in `Authorization` header.")
	docs.AddSecuritySchema("Server", "Server-Auth", "Requires a server token. Should be prefixed with `Server ` in `Authorization` header.")

	api.Setup()

	r := chi.NewRouter()

	r.Use(
		middleware.Recoverer,
		middleware.RealIP,
		middleware.CleanPath,
		corsMiddleware,
		zapchi.Logger(state.Logger, "api"),
		middleware.Timeout(30*time.Second),
	)

	routers := []uapi.APIRouter{
		// Use same order as routes folder
		auth.Router{},
		core.Router{},
		platform.Router{},
		tasks.Router{},
		users.Router{},
	}

	for _, router := range routers {
		name, desc := router.Tag()
		if name != "" {
			docs.AddTag(name, desc)
			uapi.State.SetCurrentTag(name)
		} else {
			panic("Router tag name cannot be empty")
		}

		router.Routes(r)
	}

	r.Get("/openapi", func(w http.ResponseWriter, r *http.Request) {
		w.Write(openapi)
	})

	docsTempl := template.Must(template.New("docs").Parse(docsHTML))

	r.Get("/docs", func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, "/docs/splashtail", http.StatusFound)
	})

	r.Get("/docs/{srv}", func(w http.ResponseWriter, r *http.Request) {
		var docMap = map[string]string{
			"splashtail": "/openapi",
		}

		srv := chi.URLParam(r, "srv")

		if srv == "" {
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte("Invalid service name"))
			return
		}

		v, ok := docMap[srv]

		if !ok {
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte("Invalid service"))
			return
		}

		w.Header().Set("Content-Type", "text/html; charset=utf-8")

		docsTempl.Execute(w, map[string]string{
			"url": v,
		})
	})

	// Load openapi here to avoid large marshalling in every request
	var err error
	openapi, err = json.Marshal(docs.GetSchema())

	if err != nil {
		panic(err)
	}

	r.NotFound(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte(constants.EndpointNotFound))
	})

	r.MethodNotAllowed(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusMethodNotAllowed)
		w.Write([]byte(constants.MethodNotAllowed))
	})

	return r
}
