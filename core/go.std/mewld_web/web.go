package mewld_web

import (
	"embed"
	"encoding/json"
	"fmt"
	"io"
	"io/fs"
	"net/http"
	"strconv"
	"strings"

	mconfig "github.com/cheesycod/mewld/config"
	mproc "github.com/cheesycod/mewld/proc"
	mredis "github.com/cheesycod/mewld/redis"

	"github.com/go-chi/chi/v5"
	log "github.com/sirupsen/logrus"
)

//go:embed all:ui/build
var serverRoot embed.FS
var serverRootSubbed fs.FS

var globalConfig *mconfig.CoreConfig

type WebData struct {
	RedisHandler *mredis.RedisHandler
	InstanceList *mproc.InstanceList
}

func routeStatic(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "" {
			http.Redirect(w, r, "/mewld/", http.StatusMovedPermanently)
		}

		if !strings.HasPrefix(r.URL.Path, "/api") {
			serverRoot := http.FS(serverRootSubbed)

			if strings.HasSuffix(r.URL.Path, ".js") {
				w.Header().Set("Content-Type", "application/javascript")
			} else if strings.HasSuffix(r.URL.Path, ".css") {
				w.Header().Set("Content-Type", "text/css")
			} else {
				w.Header().Set("Content-Type", "text/html; charset=utf-8")
			}

			fserve := http.FileServer(serverRoot)
			fserve.ServeHTTP(w, r)
		} else {
			// Serve API
			next.ServeHTTP(w, r)
		}
	})
}

func loginRoute(webData WebData, f func(w http.ResponseWriter, r *http.Request, sess *loginDat)) func(w http.ResponseWriter, r *http.Request) {
	return func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("X-User-ID") == "" {
			w.Write([]byte("Unauthorized. Not running under deployproxy?"))
			return
		}

		session := &loginDat{
			ID: r.Header.Get("X-User-ID"),
		}

		f(w, r, session)
	}
}

type loginDat struct {
	ID string `json:"id"`
}

// ANTIRAID-SPECIFIC: The toJson helper has been added to facilitate things
func toJson(w http.ResponseWriter, v interface{}) {
	b, err := json.Marshal(v)

	if err != nil {
		log.Error(err)
		w.WriteHeader(http.StatusInternalServerError)
		w.Write([]byte(err.Error()))
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.Write(b)
}

func CreateServer(webData WebData) *chi.Mux {
	var err error
	serverRootSubbed, err = fs.Sub(serverRoot, "ui/build")

	if err != nil {
		log.Fatal(err)
	}

	r := chi.NewRouter()

	r.Use(
		func(next http.Handler) http.Handler {
			return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				// Replace URL Path's /mewld/ with /
				r.URL.Path = strings.Replace(r.URL.Path, "/mewld", "", 1)

				next.ServeHTTP(w, r)
			})
		},
		DpAuthMiddleware,
		corsMiddleware,
		routeStatic,
	)

	globalConfig = webData.InstanceList.Config

	r.Get("/api/ping", loginRoute(
		webData,
		func(w http.ResponseWriter, r *http.Request, sess *loginDat) {
			w.Write([]byte("pong"))
		},
	))

	r.Get("/api/instance-list", loginRoute(
		webData,
		func(w http.ResponseWriter, r *http.Request, sess *loginDat) {
			toJson(w, webData.InstanceList)
		},
	))

	r.Get("/api/action-logs", loginRoute(
		webData,
		func(w http.ResponseWriter, r *http.Request, sess *loginDat) {
			payload := webData.InstanceList.Redis.LRange(webData.InstanceList.Ctx, webData.InstanceList.Config.RedisChannel+"/actlogs", 0, -1).Val()

			var payloadFinal []map[string]any

			for i, p := range payload {
				var pm map[string]any

				err := json.Unmarshal([]byte(p), &pm)

				if err != nil {
					w.WriteHeader(http.StatusBadRequest)
					w.Write([]byte(fmt.Sprintf("Could not marshal payload %d: %s", i, err.Error())))
					return
				}

				payloadFinal = append(payloadFinal, pm)
			}

			toJson(w, payloadFinal)
		},
	))

	r.Post("/api/redis/pub", loginRoute(
		webData,
		func(w http.ResponseWriter, r *http.Request, sess *loginDat) {
			payload, err := io.ReadAll(r.Body)

			if err != nil {
				log.Error(err)
				w.WriteHeader(http.StatusInternalServerError)
				w.Write([]byte("Error reading body: " + err.Error()))
				return
			}

			v := webData.InstanceList.Redis.Publish(webData.InstanceList.Ctx, webData.InstanceList.Config.RedisChannel, string(payload)).Val()

			w.Write([]byte(strconv.Itoa(int(v))))
		},
	))

	r.Get("/api/cluster-health", loginRoute(
		webData,
		func(w http.ResponseWriter, r *http.Request, sess *loginDat) {
			var cid = r.URL.Query().Get("cid")

			if cid == "" {
				cid = "0"
			}

			cInt, err := strconv.Atoi(cid)

			if err != nil {
				w.WriteHeader(http.StatusBadRequest)
				toJson(w, map[string]string{
					"error": "Invalid cid",
				})
				return
			}

			instance := webData.InstanceList.InstanceByID(cInt)

			if instance == nil {
				w.WriteHeader(http.StatusBadRequest)
				toJson(w, map[string]string{
					"error": "Invalid cid",
				})
				return
			}

			if instance.ClusterHealth == nil {
				if !instance.LaunchedFully {
					w.WriteHeader(http.StatusBadRequest)
					toJson(w, map[string]string{
						"error": "Instance not fully up",
					})
					return
				}
				ch, err := webData.InstanceList.ScanShards(instance)

				if err != nil {
					w.WriteHeader(http.StatusBadRequest)
					toJson(w, map[string]string{
						"error": "Error scanning shards: " + err.Error(),
					})
					return
				}

				instance.ClusterHealth = ch
			}

			toJson(w, map[string]any{
				"locked": instance.Locked(),
				"health": instance.ClusterHealth,
			})
		},
	))

	return r
}
