package client

import (
	"context"
	"encoding/json"
	"errors"

	"github.com/apache/pulsar-client-go/pulsar"
	"github.com/timada-org/pikav/pkg/topic"
)

type Event struct {
	UserID   string           `json:"user_id"`
	Topic    *topic.TopicName `json:"topic"`
	Name     string           `json:"name"`
	Data     any              `json:"data"`
	Metadata any              `json:"metadata"`
}

type ClientOptions struct {
	URL   string
	Topic string
	Name  string
}

type Client struct {
	Client   pulsar.Client
	producer pulsar.Producer
}

func New(options ClientOptions) (*Client, error) {
	client, err := pulsar.NewClient(pulsar.ClientOptions{
		URL: options.URL,
	})
	if err != nil {
		return nil, err
	}

	producer, err := client.CreateProducer(pulsar.ProducerOptions{
		Topic: options.Topic,
		Name:  options.Name,
	})
	if err != nil {
		return nil, err
	}

	return &Client{
		Client:   client,
		producer: producer,
	}, nil
}

func (c *Client) Send(event *Event) error {
	var err error
	var payload []byte

	if c.producer == nil {
		return errors.New("producer not initialized")
	}

	payload, err = json.Marshal(&event)
	if err != nil {
		return err
	}

	_, err = c.producer.Send(context.Background(), &pulsar.ProducerMessage{
		Payload: payload,
	})

	if err != nil {
		return err
	}

	return nil
}

func (c *Client) Close() {
	c.Client.Close()

	if c.producer != nil {
		c.producer.Close()
	}
}
