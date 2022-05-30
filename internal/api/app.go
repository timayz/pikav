package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net/http"
	"strings"

	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/core"
	"github.com/timada-org/pikav/internal/sse"
	"github.com/timada-org/pikav/pkg/topic"
)

type App struct {
	server        *sse.Server
	sender        *Sender
	config        *core.Config
	auth          *core.Auth
	client        *Client
	publishClient *Client
}

func New(config *core.Config) *App {
	var clientID string
	var client *Client
	var err error

	for i := 0; i < 15; i++ {
		clientID = fmt.Sprintf("%s-%d", config.ID, i)
		client, err = newClient(ClientOptions{
			URL:   config.Broker.URL,
			Topic: config.Broker.Topic,
			Name:  clientID,
		})

		if err == nil {
			break
		}

		if !strings.Contains(err.Error(), "is already connected to topic") {
			log.Fatalln(err)
		}
	}

	if client == nil {
		log.Fatalln(errors.New("all brokers are taken"))
	}

	publishClient, err := newClient(ClientOptions{
		URL:   config.Broker.URL,
		Topic: config.Broker.Topic,
		Name:  fmt.Sprintf("%s-pub", clientID),
	})

	if err != nil {
		log.Fatalln(err)
	}

	server := sse.New()

	sender := newSender(&SenderOptions{
		ID:     clientID,
		Topic:  config.Broker.Topic,
		client: client.Client,
		server: server,
	})

	auth, err := core.NewAuth(config.JwksURL)
	if err != nil {
		log.Fatalln(err)
	}

	app := &App{
		server,
		sender,
		config,
		auth,
		client,
		publishClient,
	}

	return app
}

func (app *App) Listen() error {
	app.sender.start()

	router := httprouter.New()
	router.GET("/sse", app.server.HandleFunc())
	router.POST("/pub", app.publish())
	router.PUT("/sub/*filter", app.subscribe())
	router.PUT("/unsub/*filter", app.unsubscribe())

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

		err = app.client.Send(&ClientEvent{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionSubscribed,
			Data: &SubEvent{
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

		err = app.client.Send(&ClientEvent{
			UserID: userID,
			Topic:  t,
			Name:   SYSSessionUnsubscribed,
			Data: &SubEvent{
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

func (app *App) publish() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		// TODO: enable when service account will be available
		// _, err := app.auth.UserID(r)
		// if err != nil {
		// 	http.Error(w, "Bad request.", http.StatusBadRequest)
		// 	return
		// }

		decoder := json.NewDecoder(r.Body)
		var input ClientEvent
		if err := decoder.Decode(&input); err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		_, err := topic.NewName(input.Topic.Value)

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		err = app.client.Send(&input)

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
