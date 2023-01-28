package api

import (
	"log"
	"net/http"

	"github.com/julienschmidt/httprouter"
	"github.com/timada-org/pikav/internal/core"
	"github.com/timada-org/pikav/internal/sse"
)

type ServerOptions struct {
	Addr    string
	JwksURL string
	Bus     *core.EventBus
	Sse     *sse.Server
}

type Server struct {
	options ServerOptions
	auth    *Auth
}

func (s *Server) Start() {
	router := httprouter.New()

	router.POST("/api/subscribe/:namespace/*filter", s.subscribe())
	router.DELETE("/api/unsubscribe/:namespace/*filter", s.unsubscribe())
	router.GET("/events", s.options.Sse.HandleFunc())

	log.Fatal(http.ListenAndServe(s.options.Addr, router))
}

func NewServer(options ServerOptions) Server {
	if options.Addr == "" {
		log.Fatalln("[server] addr is required")
	}

	if options.JwksURL == "" {
		log.Fatalln("[server] jwks url is required")
	}

	if options.Bus == nil {
		log.Fatalln("[server] bus not defined")
	}

	if options.Sse == nil {
		log.Fatalln("[server] sse server not defined")
	}

	auth, err := NewAuth(options.JwksURL)
	if err != nil {
		log.Fatalln(err)
	}

	return Server{
		options,
		auth,
	}
}

func (s *Server) subscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := s.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		clientId := s.auth.ClientID(r)
		if clientId == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := core.NewFilter(p.ByName("filter")[1:])
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		s.options.Bus.Subscribe(userID, clientId, filter)
		// app.forward(r, nil)

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}

func (s *Server) unsubscribe() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		userID, err := s.auth.UserID(r)
		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		clientID := s.auth.ClientID(r)
		if clientID == "" {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		filter, err := core.NewFilter(p.ByName("filter")[1:])

		if err != nil {
			http.Error(w, "Bad request.", http.StatusBadRequest)
			return
		}

		s.options.Bus.Unsubscribe(userID, clientID, filter)
		// app.forward(r, nil)

		w.Header().Add("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)

		if _, err := w.Write([]byte("{\"success\": true}")); err != nil {
			log.Println(err.Error())
			return
		}
	}
}
