package topic

import (
	"fmt"
	"regexp"
	"strings"
)

var topicNameRegex = regexp.MustCompile("^[^#+]+$")

type TopicName struct {
	value string
}

func NewName(value string) (*TopicName, error) {
	if value == "" {
		return nil, fmt.Errorf("topic name: %s cannot be empty", value)
	}

	if len(value) > 65535 {
		return nil, fmt.Errorf("topic name: %s cannot be have more than 65535 bytes", value)
	}

	if !topicNameRegex.MatchString(value) {
		return nil, fmt.Errorf("topic name: %s format is invalid", value)
	}

	return &TopicName{value}, nil
}

func (t *TopicName) IsServerSpecific() bool {
	return strings.HasPrefix(t.value, "$")
}
