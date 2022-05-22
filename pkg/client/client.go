package client

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/http"
	"time"

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
	URL    string
	Header http.Header
}

type Client struct {
	client  http.Client
	options ClientOptions
}

func New(options ClientOptions) (*Client, error) {

	return &Client{
		client:  http.Client{Timeout: 20 * time.Second},
		options: options,
	}, nil
}

func (c *Client) Send(event *Event) error {
	payload, err := json.Marshal(&event)
	if err != nil {
		return err
	}

	req, err := http.NewRequest("POST", c.options.URL, bytes.NewBuffer(payload))

	if err != nil {
		return err
	}

	if c.options.Header != nil {
		req.Header = c.options.Header
	}

	req.Header.Add("Content-Type", "application/json")

	resp, err := c.client.Do(req)

	if err != nil {
		return err
	}

	if resp.StatusCode < 300 {
		return nil
	}
	defer resp.Body.Close()

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	return fmt.Errorf("%s %s", resp.Status, body)
}
