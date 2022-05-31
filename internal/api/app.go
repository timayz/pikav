package api

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"

	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/core"
	"github.com/timada-org/pikav/internal/sse"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
)

type App struct {
	server *sse.Server
	bus    *core.EventBus
	config *core.Config
	auth   *Auth
	nodes  []*client.Client
}

func New(config *core.Config) *App {
	var err error

	server := sse.New()

	bus := core.NewEventBus(&core.EventBusOptions{
		Server: server,
	})

	auth, err := NewAuth(config.JwksURL)
	if err != nil {
		log.Fatalln(err)
	}

	nodes := []*client.Client{}
	for _, node := range config.Nodes {
		nodes = append(nodes, client.New(client.ClientOptions{Zone: node.Zone, URL: node.URL, Shared: node.Shared}))
	}

	app := &App{
		server,
		bus,
		config,
		auth,
		nodes,
	}

	return app
}

func (app *App) Listen() error {
	router := httprouter.New()
	router.GET("/sse", app.server.HandleFunc())
	router.POST("/pub", app.publish())
	router.PUT("/sub/*filter", app.subscribe())
	router.PUT("/unsub/*filter", app.unsubscribe())

	log.Printf("Listening on %s", app.config.Addr)

	return http.ListenAndServe(app.config.Addr, router)
}

func (app *App) subscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		sessionId := app.auth.SessionID(r)
		if sessionId == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := topic.NewFilter(p.ByName("filter")[1:])
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		app.bus.Subscribe(userID, sessionId, filter)
		app.forward(r, nil)

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

func (app *App) unsubscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		sessionId := app.auth.SessionID(r)
		if sessionId == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := topic.NewFilter(p.ByName("filter")[1:])

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		app.bus.Unsubscribe(userID, sessionId, filter)
		app.forward(r, nil)

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

func (app *App) publish() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		// TODO: enable when service account will be available
		// _, err := app.auth.UserID(r)
		// if err != nil {
		// 	http.Error(w, "Bad request.", http.StatusBadRequest)
		// 	return
		// }

		decoder := json.NewDecoder(r.Body)
		var input core.Event
		if err := decoder.Decode(&input); err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		_, err := topic.NewName(input.Topic.Value)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		app.bus.Send(&input)

		payload, err := json.Marshal(&input)
		if err != nil {
			http.Error(w, "Internal server error.", http.StatusInternalServerError)
			log.Println(err.Error())
			return
		}

		app.forward(r, payload)

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

func (app *App) forward(r *http.Request, body []byte) {
	for _, node := range app.nodes {

		go func(client *client.Client) {
			var rb io.Reader

			if body != nil {
				rb = bytes.NewBuffer(body)
			}

			if err := client.Forward(r, rb); err != nil {
				log.Println(err.Error())
			}
		}(node)
	}
}
