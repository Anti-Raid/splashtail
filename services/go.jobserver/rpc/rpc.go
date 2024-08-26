package rpc

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/infinitybotlist/eureka/jsonimpl"
	"go.jobserver/core"
	"go.jobserver/rpc_messages"
	"go.jobserver/state"
)

func JobserverRpcServer() {
	handler := http.NewServeMux()

	handler.Handle("/", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("jobserver"))
	}))

	handler.HandleFunc("/spawn-task", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
			return
		}

		// Read request
		var spawnTask rpc_messages.SpawnTask

		err := jsonimpl.UnmarshalReader(r.Body, &spawnTask)

		if err != nil {
			http.Error(w, fmt.Sprintf("Error reading request: %s", err), http.StatusBadRequest)
			return
		}

		// Spawn task
		resp, err := core.SpawnTask(spawnTask)

		if err != nil {
			http.Error(w, fmt.Sprintf("Error spawning task: %s", err), http.StatusInternalServerError)
			return
		}

		// Write response
		err = jsonimpl.MarshalToWriter(w, resp)

		if err != nil {
			http.Error(w, fmt.Sprintf("Error writing response: %s", err), http.StatusInternalServerError)
			return
		}
	})

	// Start server
	err := http.ListenAndServe(":"+strconv.Itoa(state.Config.BasePorts.Jobserver.Parse()+int(state.ClusterID)), handler)

	if err != nil {
		panic(err)
	}
}
