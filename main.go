package main

import (
	"fmt"
	"html/template"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"runtime"
	"strconv"
	"strings"
	"syscall"
	"time"

	mconfig "github.com/cheesycod/mewld/config"
	mloader "github.com/cheesycod/mewld/loader"
	mproc "github.com/cheesycod/mewld/proc"
	mutils "github.com/cheesycod/mewld/utils"
	"gopkg.in/yaml.v3"

	"github.com/anti-raid/splashtail/api"
	"github.com/anti-raid/splashtail/bgtasks"
	"github.com/anti-raid/splashtail/config"
	"github.com/anti-raid/splashtail/constants"
	"github.com/anti-raid/splashtail/jobserver"
	"github.com/anti-raid/splashtail/mewld_web"
	"github.com/anti-raid/splashtail/routes/auth"
	"github.com/anti-raid/splashtail/routes/core"
	"github.com/anti-raid/splashtail/routes/platform"
	"github.com/anti-raid/splashtail/routes/tasks"
	"github.com/anti-raid/splashtail/state"
	"github.com/anti-raid/splashtail/types"

	"github.com/cloudflare/tableflip"
	docs "github.com/infinitybotlist/eureka/doclib"
	"github.com/infinitybotlist/eureka/uapi"
	"go.uber.org/zap"

	"github.com/infinitybotlist/eureka/zapchi"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"

	_ "embed"

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

func main() {
	state.Setup()

	if len(os.Args) < 2 {
		os.Args = append(os.Args, "help")
	}

	state.CurrentOperationMode = os.Args[1]

	switch os.Args[1] {
	case "webserver":
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

		// Load mewld bot
		mldF, err := os.ReadFile("mewld-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		mldConfig.Token = state.Config.DiscordAuth.Token
		mldConfig.Oauth = mconfig.Oauth{
			ClientID:     state.Config.DiscordAuth.ClientID,
			ClientSecret: state.Config.DiscordAuth.ClientSecret,
			RedirectURL:  state.Config.DiscordAuth.MewldRedirect,
		}

		if mldConfig.Redis != state.Config.Meta.RedisURL.Parse() {
			panic("Redis URL in mewld.yaml does not match the one in config.yaml")
		}

		il, rh, err := mloader.Load(&mldConfig, &mproc.LoaderData{
			Start: func(l *mproc.InstanceList, i *mproc.Instance, cm *mproc.ClusterMap) error {
				var cmd *exec.Cmd
				if l.Config.Interp != "" {
					cmd = exec.Command(
						l.Config.Interp,
						l.Dir+"/"+l.Config.Module,
						mutils.ToPyListUInt64(i.Shards),
						mutils.UInt64ToString(l.ShardCount),
						strconv.Itoa(i.ClusterID),
						cm.Name,
						l.Dir,
						strconv.Itoa(len(l.Map)),
						state.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
					)
				} else {
					cmd = exec.Command(
						l.Dir+"/"+l.Config.Module,
						mutils.ToPyListUInt64(i.Shards),
						mutils.UInt64ToString(l.ShardCount),
						strconv.Itoa(i.ClusterID),
						cm.Name,
						l.Dir,
						strconv.Itoa(len(l.Map)),
						state.Config.Sites.API.Parse(),
						l.Config.RedisChannel,
					)
				}

				cmd.Stdout = os.Stdout
				cmd.Stderr = os.Stderr

				env := os.Environ()

				env = append(env, "MEWLD_CHANNEL="+l.Config.RedisChannel)
				env = append(env, "REDIS_URL="+state.Config.Meta.RedisURL.Parse())

				cmd.Env = env
				cmd.Dir = l.Dir

				i.Command = cmd

				// Spawn process
				return cmd.Start()
			},
		})

		if err != nil {
			panic(err)
		}

		defer func() {
			a := recover()

			if a != nil {
				il.KillAll()
			}
		}()

		state.MewldInstanceList = il

		r.Mount("/mewld", mewld_web.CreateServer(mewld_web.WebData{
			RedisHandler: rh,
			InstanceList: il,
		}))

		// If GOOS is windows, do normal http server
		if runtime.GOOS == "linux" || runtime.GOOS == "darwin" {
			upg, _ := tableflip.New(tableflip.Options{})
			defer upg.Stop()

			go func() {
				sig := make(chan os.Signal, 1)
				signal.Notify(sig, syscall.SIGHUP)
				for range sig {
					state.Logger.Info("Received SIGHUP, upgrading server")
					upg.Upgrade()
				}
			}()

			// Listen must be called before Ready
			ln, err := upg.Listen("tcp", state.Config.Meta.Port.Parse())

			if err != nil {
				il.KillAll()
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
			err = http.ListenAndServe(state.Config.Meta.Port.Parse(), r)

			if err != nil {
				il.KillAll()
				state.Logger.Fatal("Error binding to socket", zap.Error(err))
			}
		}
	case "jobs":
		mldF, err := os.ReadFile("mewld-" + config.CurrentEnv + ".yaml")

		if err != nil {
			panic(err)
		}

		var mldConfig mconfig.CoreConfig

		err = yaml.Unmarshal(mldF, &mldConfig)

		if err != nil {
			panic(err)
		}

		state.MewldInstanceList = &mproc.InstanceList{
			Config: &mldConfig,
		}

		// Set state of all pending tasks to 'failed'
		_, err = state.Pool.Exec(state.Context, "UPDATE tasks SET state = $1 WHERE state = $2", "failed", "pending")

		if err != nil {
			panic(err)
		}

		// Load jobs
		bgtasks.StartAllTasks()

		// Start IPC
		jobserver.Start()

		// This should never return
		os.Exit(1)
	default:
		fmt.Println("Splashtail Usage: splashtail <component>")
		fmt.Println("webserver: Starts the webserver")
		fmt.Println("jobs: Starts the jobserver (currently includes IPC as well)")
		os.Exit(1)
	}
}
