package sse

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
)

type Session struct {
	messageChan chan string
}

func newSession() *Session {
	return &Session{
		messageChan: make(chan string),
	}
}

func (s *Session) Send(e *Event) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)

	if err := enc.Encode(e); err != nil {
		log.Println(err)
	}

	if s.messageChan == nil {
		return
	}

	go func() {
		s.messageChan <- buf.String()
	}()
}

func (s *Session) listen(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	defer func() {
		close(s.messageChan)
		s.messageChan = nil
	}()

	flusher, _ := w.(http.Flusher)

	for {
		select {

		// message will received here and printed
		case message := <-s.messageChan:
			fmt.Fprintf(w, "data: %s\n\n", message)
			flusher.Flush()

		// connection is closed then defer will be executed
		case <-r.Context().Done():
			return

		}
	}
}
