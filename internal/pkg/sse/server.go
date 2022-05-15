package sse

import (
	"net/http"
	"sync"

	"github.com/julienschmidt/httprouter"
	gonanoid "github.com/matoous/go-nanoid/v2"
)

type Server struct {
	mux                 sync.RWMutex
	NewSessionHandler   func(id string, session *Session)
	CloseSessionHandler func(id string, session *Session)
	sessions            map[string]*Session
}

func New() *Server {
	return &Server{
		sessions: make(map[string]*Session),
	}
}

func (s *Server) HandleFunc() httprouter.Handle {
	return func(w http.ResponseWriter, r *http.Request, p httprouter.Params) {
		id, err := gonanoid.New()

		if err != nil {
			return
		}

		session := newSession()

		s.mux.Lock()
		s.sessions[id] = session
		s.mux.Unlock()

		if s.NewSessionHandler != nil {
			s.NewSessionHandler(id, session)
		}

		session.Send(&Event{
			Topic: SYSSessionTopic,
			Name:  SYSSessionCreated,
			Data:  id,
		})

		session.listen(w, r)

		s.mux.Lock()
		delete(s.sessions, id)
		s.mux.Unlock()

		if s.CloseSessionHandler != nil {
			s.CloseSessionHandler(id, session)
		}
	}
}

func (s *Server) Get(id string) (*Session, bool) {
	s.mux.RLock()
	defer s.mux.RUnlock()

	session, ok := s.sessions[id]

	return session, ok
}
