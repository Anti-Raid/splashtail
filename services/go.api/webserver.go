package main

import (
	"html/template"
	"net/http"
	"os"
	"os/signal"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"

	_ "embed"

	"github.com/cloudflare/tableflip"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/jsonimpl"
	"github.com/infinitybotlist/eureka/uapi"
	"github.com/infinitybotlist/eureka/zapchi"
	"go.api/api"
	"go.api/constants"
	"go.api/integrations/gitlogs"
	"go.api/routes/auth"
	"go.api/routes/core"
	"go.api/routes/guilds"
	"go.api/routes/jobs"
	"go.api/routes/platform"
	"go.api/routes/users"
	"go.api/state"
	"go.api/types"
	"go.uber.org/zap"
)

//go:embed docs/docs.html
var docsHTML string

//go:embed docs/desc.md
var descMd string

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
		w.Header().Set("access-control-expose-headers", "Bucket, Retry-After, Req-Limit, Req-Made, X-Error-Type")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE")

		if r.Method == "OPTIONS" {
			_, _ = w.Write([]byte{})
			return
		}

		w.Header().Set("Content-Type", "application/json")

		next.ServeHTTP(w, r)
	})
}

func CreateWebserver() *chi.Mux {
	docs.DocsSetupData = &docs.SetupData{
		URL:         state.Config.Sites.API,
		ErrorStruct: types.ApiError{},
		Info: docs.Info{
			Title:          "Antiraid API",
			TermsOfService: "https://antiraid.xyz/terms",
			Version:        "7.0",
			Description:    descMd,
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
		guilds.Router{},
		jobs.Router{},
		platform.Router{},
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
		_, _ = w.Write(openapi)
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
			_, _ = w.Write([]byte("Invalid service name"))
			return
		}

		v, ok := docMap[srv]

		if !ok {
			w.WriteHeader(http.StatusBadRequest)
			_, _ = w.Write([]byte("Invalid service"))
			return
		}

		w.Header().Set("Content-Type", "text/html; charset=utf-8")

		err := docsTempl.Execute(w, map[string]string{
			"url": v,
		})

		if err != nil {
			state.Logger.Error("Error executing template", zap.Error(err))
			w.WriteHeader(http.StatusInternalServerError)
			_, _ = w.Write([]byte("Error executing template"))
		}
	})

	// Load openapi here to avoid large marshalling in every request
	var err error
	openapi, err = jsonimpl.Marshal(docs.GetSchema())

	if err != nil {
		panic(err)
	}

	r.NotFound(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		_, _ = w.Write([]byte(constants.EndpointNotFound))
	})

	r.MethodNotAllowed(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusMethodNotAllowed)
		_, _ = w.Write([]byte(constants.MethodNotAllowed))
	})

	// Mount integrations
	r.Mount("/integrations/gitlogs", gitlogs.Setup())

	return r
}

// Launches the webserver
func main() {
	state.Setup()

	state.CurrentOperationMode = "webserver"

	r := CreateWebserver()

	// If GOOS is windows, do normal http server
	if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
		upg, _ := tableflip.New(tableflip.Options{})
		defer upg.Stop()

		go func() {
			sig := make(chan os.Signal, 1)
			signal.Notify(sig, syscall.SIGHUP)
			for range sig {
				state.Logger.Info("Received SIGHUP, upgrading server")
				err := upg.Upgrade()

				if err != nil {
					state.Logger.Error("Error upgrading server", zap.Error(err))
				}
			}
		}()

		// Listen must be called before Ready
		ln, err := upg.Listen("tcp", ":"+strconv.Itoa(state.Config.Meta.Port))

		if err != nil {
			state.Logger.Fatal("Error binding to socket", zap.Error(err))
		}

		defer ln.Close()

		server := http.Server{
			ReadTimeout: 30 * time.Second,
			Handler:     r,
		}

		go func() {
			err := server.Serve(ln)
			if err != http.ErrServerClosed {
				state.Logger.Error("Server failed due to unexpected error", zap.Error(err))
			}
		}()

		if err := upg.Ready(); err != nil {
			state.Logger.Fatal("Error calling upg.Ready", zap.Error(err))
		}

		<-upg.Exit()
	} else {
		// Tableflip not supported
		state.Logger.Warn("Tableflip not supported on this platform, this is not a production-capable server.")
		err := http.ListenAndServe(":"+strconv.Itoa(state.Config.Meta.Port), r)

		if err != nil {
			state.Logger.Fatal("Error binding to socket", zap.Error(err))
		}
	}
}
