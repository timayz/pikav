package core

import (
	"sync"

	"github.com/timada-org/pikav/internal/sse"
	"github.com/timada-org/pikav/pkg/topic"
)

type Event struct {
	UserID   string           `json:"user_id"`
	Topic    *topic.TopicName `json:"topic"`
	Name     string           `json:"name"`
	Data     any              `json:"data"`
	Metadata any              `json:"metadata"`
}

type EventBusOptions struct {
	Server *sse.Server
}

type EventBus struct {
	mux           sync.RWMutex
	server        *sse.Server
	subscriptions map[string]map[string]*Subscription
	userIds       map[string]string
}

func NewEventBus(options *EventBusOptions) *EventBus {
	bus := &EventBus{
		subscriptions: make(map[string]map[string]*Subscription),
		userIds:       make(map[string]string),
		server:        options.Server,
	}

	options.Server.CloseSessionHandler = func(id string, session *sse.Session) {
		bus.mux.Lock()
		defer bus.mux.Unlock()

		userID, ok := bus.userIds[id]
		if !ok {
			return
		}

		delete(bus.userIds, id)
		delete(bus.subscriptions[userID], id)

		if len(bus.subscriptions[userID]) == 0 {
			delete(bus.subscriptions, userID)
		}
	}

	return bus
}

func (bus *EventBus) Send(event *Event) {
	bus.mux.RLock()
	defer bus.mux.RUnlock()

	subscriptions, ok := bus.subscriptions[event.UserID]

	if !ok {
		return
	}

	for _, session := range subscriptions {
		session.send(event)
	}
}

func (bus *EventBus) Subscribe(userID string, sessionID string, filter *topic.TopicFilter) {
	session, ok := bus.server.Get(sessionID)
	if !ok {
		return
	}

	bus.mux.Lock()
	defer bus.mux.Unlock()

	if _, ok := bus.subscriptions[userID]; !ok {
		bus.subscriptions[userID] = make(map[string]*Subscription)
	}

	if oldUserID, ok := bus.userIds[sessionID]; !ok || oldUserID != userID {
		delete(bus.userIds, sessionID)
		delete(bus.subscriptions[oldUserID], sessionID)

		bus.userIds[sessionID] = userID
		bus.subscriptions[userID][sessionID] = &Subscription{
			session: session, filters: make(map[string]*topic.TopicFilter),
		}
	}

	bus.subscriptions[userID][sessionID].add(filter)

}

func (bus *EventBus) Unsubscribe(userID string, sessionID string, filter *topic.TopicFilter) {
	bus.mux.RLock()
	defer bus.mux.RUnlock()

	if _, ok := bus.server.Get(sessionID); !ok {
		return
	}

	if _, ok := bus.subscriptions[userID]; !ok {
		return
	}

	if subscription, ok := bus.subscriptions[userID][sessionID]; ok {
		subscription.remove(filter)
	}
}

type Subscription struct {
	mux     sync.RWMutex
	filters map[string]*topic.TopicFilter
	session *sse.Session
}

func (s *Subscription) add(filter *topic.TopicFilter) {
	s.mux.Lock()
	defer s.mux.Unlock()
	s.filters[filter.Value] = filter
}

func (s *Subscription) remove(filter *topic.TopicFilter) {
	s.mux.Lock()
	defer s.mux.Unlock()
	delete(s.filters, filter.Value)
}

func (s *Subscription) send(event *Event) {
	s.mux.RLock()
	defer s.mux.RUnlock()

	for _, filter := range s.filters {
		if filter.Match(event.Topic) {
			s.session.Send(&sse.Event{
				Topic:    event.Topic.Value,
				Name:     event.Name,
				Data:     event.Data,
				Metadata: event.Metadata,
			})
		}
	}
}
