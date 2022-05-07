package sse

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"time"

	gonanoid "github.com/matoous/go-nanoid/v2"
)

type Event struct {
	Topic    string `json:"topic"`
	Name     string `json:"name"`
	Data     any    `json:"data"`
	Metadata any    `json:"metadata"`
}

type Session struct {
	id      string
	writer  *bufio.Writer
	message chan string
}

func newSession(w *bufio.Writer) (*Session, error) {
	id, err := gonanoid.New()
	if err != nil {
		return nil, err
	}

	return &Session{
		id:      id,
		writer:  w,
		message: make(chan string),
	}, nil
}

func (s *Session) send(e *Event) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)

	go func() {
		if err := enc.Encode(e); err != nil {
			s.message <- ""
		} else {
			s.message <- buf.String()
		}
	}()
}

func (s *Session) start() {
	go func() {
		for {
			time.Sleep(10 * time.Second)
			fmt.Fprint(s.writer, "ping\n\n")

			if err := s.writer.Flush(); err != nil {
				s.message <- ""
				break
			}
		}
	}()

	for {
		message := <-s.message
		if message == "" {
			break
		}

		fmt.Fprintf(s.writer, "data: %s\n\n", message)

		if err := s.writer.Flush(); err != nil {
			break
		}
	}
}
