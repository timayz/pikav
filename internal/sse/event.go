package sse

type Event struct {
	Topic    string `json:"topic"`
	Name     string `json:"name"`
	Data     any    `json:"data"`
	Metadata any    `json:"metadata"`
}

const SYSSessionTopic = "$SYS/session"

const (
	SYSSessionCreated = "Created"
)
