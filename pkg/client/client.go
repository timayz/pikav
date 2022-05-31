package client

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"time"

	"github.com/timada-org/pikav/internal/core"
)

type ClientOptions struct {
	Zone   string
	Shared bool
	URL    string
	Header http.Header
}

type Client struct {
	client         http.Client
	options        ClientOptions
	publishURL     string
	subscribeURL   string
	unsubscribeURL string
}

func New(options ClientOptions) *Client {
	return &Client{
		client:         http.Client{Timeout: 20 * time.Second},
		options:        options,
		publishURL:     fmt.Sprintf("%s/pub", options.URL),
		subscribeURL:   fmt.Sprintf("%s/sub", options.URL),
		unsubscribeURL: fmt.Sprintf("%s/unsub", options.URL),
	}
}

func (c *Client) Shared() bool {
	return c.options.Shared
}

func (c *Client) Send(event *core.Event) error {
	payload, err := json.Marshal(&event)
	if err != nil {
		return err
	}

	req, err := http.NewRequest("POST", c.publishURL, bytes.NewBuffer(payload))

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

func (c *Client) Forward(req *http.Request, b io.Reader) error {
	if _, ok := req.Header["X-Pikav-Forwarded-By"]; ok {
		return nil
	}

	newReq, err := http.NewRequest(req.Method, fmt.Sprintf("%s%s", c.options.URL, req.RequestURI), b)

	if err != nil {
		return err
	}

	newReq.Header.Set("X-Pikav-Forwarded-By", c.options.Zone)
	newReq.Header.Set("Authorization", req.Header.Get("Authorization"))
	newReq.Header.Set("X-Pikav-Session-ID", req.Header.Get("X-Pikav-Session-ID"))

	resp, err := c.client.Do(newReq)

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
