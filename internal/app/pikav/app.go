package pikav

import (
	"log"
	"net/http"

	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/pkg/core"
	"github.com/timada-org/pikav/internal/pkg/sse"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
)

type App struct {
	server *sse.Server
	sender *Sender
	config *core.Config
	auth   *core.Auth
	client *client.Client
}

func New() *App {
	config, err := core.NewConfig()
	if err != nil {
		log.Fatalln(err)
	}

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

	auth, err := core.NewAuth(config.JwksURL)
	if err != nil {
		log.Fatalln(err)
	}

	app := &App{
		server: server,
		sender: sender,
		config: config,
		auth:   auth,
		client: c,
	}

	return app
}

func (app *App) Listen() error {
	app.sender.start()

	router := httprouter.New()
	router.GET("/sse", app.server.HandleFunc())
	router.PUT("/subscribe/*filter", app.subscribe())
	router.PUT("/unsubscribe/*filter", app.unsubscribe())

	log.Printf("Listening on %s", app.config.Addr)

	return http.ListenAndServe(app.config.Addr, router)
}

func (app *App) Close() {
	app.client.Close()
	app.sender.Close()
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

		t, err := topic.NewName(sse.SYSSessionTopic)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		err = app.client.Send(&client.Event{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionSubscribed,
			Data: &SubscribeEvent{
				SessionId: sessionId,
				Filter:    *filter,
			},
		})

		w.Header().Add("Content-Type", "application/json")

		if err != nil {
			log.Println(err.Error())
			http.Error(w, "{\"success\": false}", http.StatusInternalServerError)
			return
		}

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

		t, err := topic.NewName(sse.SYSSessionTopic)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		err = app.client.Send(&client.Event{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionUnsubscribed,
			Data: &SubscribeEvent{
				SessionId: sessionId,
				Filter:    *filter,
			},
		})

		w.Header().Add("Content-Type", "application/json")

		if err != nil {
			log.Println(err.Error())
			http.Error(w, "{\"success\": false}", http.StatusInternalServerError)
			return
		}

		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}
