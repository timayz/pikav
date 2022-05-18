package pikav

import (
	"context"
	"encoding/json"
	"log"
	"strings"
	"sync"

	"github.com/apache/pulsar-client-go/pulsar"
	"github.com/mitchellh/mapstructure"
	"github.com/timada-org/pikav/internal/pkg/sse"
	"github.com/timada-org/pikav/pkg/client"
	"github.com/timada-org/pikav/pkg/topic"
)

type SenderOptions struct {
	ID     string
	Topic  string
	client pulsar.Client
	server *sse.Server
}

type Sender struct {
	mux           sync.RWMutex
	consumer      pulsar.Consumer
	server        *sse.Server
	subscriptions map[string]map[string]*Subscription
	userIds       map[string]string
}

func newSender(options *SenderOptions) *Sender {
	consumer, err := options.client.Subscribe(pulsar.ConsumerOptions{
		Topic:            options.Topic,
		SubscriptionName: options.ID,
		Type:             pulsar.Exclusive,
	})
	if err != nil {
		log.Fatalf("Failed to create JWKS from resource at the given URL.\nError: %s", err.Error())
	}

	sender := &Sender{
		consumer:      consumer,
		subscriptions: make(map[string]map[string]*Subscription),
		userIds:       make(map[string]string),
		server:        options.server,
	}

	options.server.CloseSessionHandler = func(id string, session *sse.Session) {
		sender.mux.Lock()
		defer sender.mux.Unlock()

		userID, ok := sender.userIds[id]
		if !ok {
			return
		}

		delete(sender.userIds, id)
		delete(sender.subscriptions[userID], id)

		if len(sender.subscriptions[userID]) == 0 {
			delete(sender.subscriptions, userID)
		}
	}

	return sender
}

func (d *Sender) start() {
	go func() {
		for {
			msg, err := d.consumer.Receive(context.Background())
			if err != nil {
				log.Fatal(err)
			}

			var event client.Event
			if err := json.Unmarshal(msg.Payload(), &event); err != nil {
				log.Println(err)
				continue
			}

			if strings.HasPrefix(event.Topic.Value, "$SYS") {
				d.handleSys(&event)
				continue
			}

			d.handle(&event)
		}
	}()
}

func (d *Sender) handle(event *client.Event) {
	var data SubEvent
	if err := mapstructure.Decode(event.Data, &data); err != nil {
		log.Println(err)
		return
	}

	d.mux.RLock()
	defer d.mux.RUnlock()

	subscriptions, ok := d.subscriptions[event.UserID]

	if !ok {
		return
	}

	for _, session := range subscriptions {
		session.send(event)
	}
}

func (d *Sender) handleSys(event *client.Event) {
	switch event.Topic.Value {
	case sse.SYSSessionTopic:
		switch event.Name {
		case SYSSessionSubscribed:
			d.handleSysSessionSubscribed(event)
		case SYSSessionUnsubscribed:
			d.handleSysSessionUnsubscribed(event)
		}
	}
}

func (d *Sender) handleSysSessionSubscribed(event *client.Event) {
	var data SubEvent
	if err := mapstructure.Decode(event.Data, &data); err != nil {
		log.Println(err)
		return
	}

	d.mux.Lock()
	defer d.mux.Unlock()

	session, ok := d.server.Get(data.SessionId)
	if !ok {
		return
	}

	if _, ok := d.subscriptions[event.UserID]; !ok {
		d.subscriptions[event.UserID] = make(map[string]*Subscription)
	}

	if userID, ok := d.userIds[data.SessionId]; !ok || userID != event.UserID {
		delete(d.userIds, data.SessionId)
		delete(d.subscriptions[userID], data.SessionId)

		d.userIds[data.SessionId] = event.UserID
		d.subscriptions[event.UserID][data.SessionId] = &Subscription{
			session: session, filters: make(map[string]*topic.TopicFilter),
		}
	}

	d.subscriptions[event.UserID][data.SessionId].add(&data.Filter)
}

func (d *Sender) handleSysSessionUnsubscribed(event *client.Event) {
	var data SubEvent
	if err := mapstructure.Decode(event.Data, &data); err != nil {
		log.Println(err)
		return
	}

	d.mux.RLock()
	defer d.mux.RUnlock()

	if _, ok := d.server.Get(data.SessionId); !ok {
		return
	}

	if _, ok := d.subscriptions[event.UserID]; !ok {
		return
	}

	if subscription, ok := d.subscriptions[event.UserID][data.SessionId]; ok {
		subscription.remove(&data.Filter)
	}
}

func (d *Sender) Close() {
	d.consumer.Close()
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

func (s *Subscription) send(event *client.Event) {
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
