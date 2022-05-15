package pikav

import "github.com/timada-org/pikav/pkg/topic"

type SubscribeEvent struct {
	SessionId string            `mapstructure:"session_id" json:"session_id"`
	Filter    topic.TopicFilter `mapstructure:"filter" json:"filter"`
}

const (
	SYSSessionSubscribed   = "Subscribed"
	SYSSessionUnsubscribed = "Unsubscribed"
)
