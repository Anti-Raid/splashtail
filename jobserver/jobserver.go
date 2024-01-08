package jobserver

import (
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/anti-raid/splashtail/jobserver/core"
	"github.com/anti-raid/splashtail/jobserver/endpoints/create_task"
	"github.com/anti-raid/splashtail/jobserver/endpoints/execute_task"
	"github.com/anti-raid/splashtail/state"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/infinitybotlist/eureka/zapchi"

	jsoniter "github.com/json-iterator/go"
	"go.uber.org/zap"
)

var expectedSecretMap map[string]string // Set during setup

var json = jsoniter.ConfigFastest

var ipcEvents = map[string]core.IPC{
	"create_task":  create_task.CreateTask,
	"execute_task": execute_task.ExecuteTask,
}

type IpcRequest struct {
	// Arguments to pass to the ipc command
	Args map[string]any `json:"args"`
}

// Auth header format: client_type secret
func identifyClient(r *http.Request) (string, error) {
	if r.Header.Get("Authorization") == "" {
		return "", fmt.Errorf("no authorization header provided")
	}

	authSplit := strings.Split(r.Header.Get("Authorization"), " ")

	if len(authSplit) != 2 {
		return "", fmt.Errorf("invalid authorization header provided [not of format client_type secret]")
	}

	clientType := authSplit[0]
	secret := authSplit[1]

	// Verify secret
	expectedSecret, ok := expectedSecretMap[clientType]

	if !ok {
		return "", fmt.Errorf("invalid client type provided")
	}

	if secret != expectedSecret {
		return "", fmt.Errorf("invalid secret provided for client type %s", clientType)
	}

	return clientType, nil
}

func Start() {
	expectedSecretMap = state.Config.Meta.JobServerSecrets.Parse()

	r := chi.NewMux()

	r.Use(
		middleware.Recoverer,
		zapchi.Logger(state.Logger, "jobserver_http"),
		middleware.Timeout(120*time.Second),
	)

	r.Post("/ipc/{name}", func(w http.ResponseWriter, r *http.Request) {
		clientType, err := identifyClient(r)

		if err != nil {
			w.Write([]byte("identifyClient failed:" + err.Error()))
			w.WriteHeader(http.StatusForbidden)
			return
		}

		name := chi.URLParam(r, "name")

		if name == "" {
			w.Write([]byte("IPC name must be provided"))
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		ipc, ok := ipcEvents[name]

		if !ok {
			w.Write([]byte("IPC does not exist"))
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		var req *IpcRequest

		err = json.NewDecoder(r.Body).Decode(&req)

		if err != nil {
			w.Write([]byte("Invalid JSON"))
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		if len(req.Args) == 0 {
			w.Write([]byte("No args provided"))
			w.WriteHeader(http.StatusBadRequest)
			return
		}

		resp, err := ipc.Exec(clientType, req.Args)

		if err != nil {
			w.Write([]byte(err.Error()))
			w.WriteHeader(http.StatusInternalServerError)
			return
		}

		if len(resp) == 0 {
			w.WriteHeader(http.StatusNoContent)
			return
		}

		err = json.NewEncoder(w).Encode(resp)

		if err != nil {
			w.Write([]byte(err.Error()))
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
	})

	err := http.ListenAndServe(state.Config.Meta.JobServerPort.Parse(), r)

	if err != nil {
		state.Logger.Fatal("Failed to start job server", zap.Error(err))
	}
}
