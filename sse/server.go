package sse

import (
	"bufio"
	"sync"

	"github.com/gofiber/fiber/v2"
	"github.com/valyala/fasthttp"
)

type Context struct {
	mu       sync.RWMutex
	sessions map[string]Session
}

func (c *Context) add(s *Session) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.sessions[s.id] = *s
	s.send(&Event{
		Topic: "$SYS/session",
		Name:  "created",
		Data:  s.id,
	})
}

func (c *Context) remove(s *Session) {
	c.mu.Lock()
	defer c.mu.Unlock()
	delete(c.sessions, s.id)
}

type Server struct {
	context Context
}

var server Server

func init() {
	server = Server{
		context: Context{
			sessions: make(map[string]Session),
		},
	}
}

func Handler(c *fiber.Ctx) error {
	c.Set("Content-Type", "text/event-stream")
	c.Set("Cache-Control", "no-cache")
	c.Set("Connection", "keep-alive")
	c.Set("Transfer-Encoding", "chunked")

	c.Context().SetBodyStreamWriter(fasthttp.StreamWriter(func(w *bufio.Writer) {
		if session, err := newSession(w); err == nil {
			server.context.add(session)
			defer server.context.remove(session)
			session.start()
		}
	}))

	return nil
}
