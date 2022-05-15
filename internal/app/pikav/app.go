package pikav

import (
	"log"
	"net/http"

	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/pkg/sse"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
)

type App struct {
	server *sse.Server
	sender *Sender
	config *Config
	auth   *Auth
	client *client.Client
}

func New() *App {
	config := NewConfig()

	c, err := client.New(client.ClientOptions{
		URL:   config.Broker.URL,
		Topic: config.Broker.Topic,
		Name:  config.ID,
	})

	if err != nil {
		log.Fatalln(err)
	}

	server := sse.New()

	sender := newSender(&SenderOptions{
		ID:     config.ID,
		Topic:  config.Broker.Topic,
		client: c.Client,
		server: server,
	})

	app := &App{
		server: server,
		sender: sender,
		config: config,
		auth:   newAuth(config.JwksURL),
		client: c,
	}

	return app
}

func (app *App) Listen() error {
	app.sender.start()

	router := httprouter.New()
	router.GET("/sse", app.server.HandleFunc())
	router.PUT("/subscribe/:filter", app.subscribe())
	router.PUT("/unsubscribe/:filter", app.unsubscribe())

	log.Printf("Listening on %s", app.config.Addr)

	return http.ListenAndServe(app.config.Addr, router)
}

func (app *App) Close() {
	app.client.Close()
	app.sender.Close()
}

func (app *App) subscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.userID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		sessionId := app.auth.sessionID(r)
		if sessionId == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := topic.NewFilter(p.ByName("filter"))

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		t, err := topic.NewName(sse.SYSSessionTopic)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		app.client.Send(&client.Event{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionSubscribed,
			Data: &SubscribeEvent{
				SessionId: sessionId,
				Filter:    *filter,
			},
		})

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("{\"success\": true}"))
	}
}

func (app *App) unsubscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := app.auth.userID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		sessionId := app.auth.sessionID(r)
		if sessionId == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := topic.NewFilter(p.ByName("filter"))

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		t, err := topic.NewName(sse.SYSSessionTopic)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		app.client.Send(&client.Event{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionUnsubscribed,
			Data: &SubscribeEvent{
				SessionId: sessionId,
				Filter:    *filter,
			},
		})

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("{\"success\": true}"))
	}
}
