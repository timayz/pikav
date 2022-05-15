package topic

import (
	"fmt"
	"regexp"
	"strings"
)

var topicFilterRegex = regexp.MustCompile(`^(([^+#]*|\+)(/([^+#]*|\+))*(/#)?|#)$`)

type TopicFilter struct {
	Value string `json:"value"`
}

func NewFilter(value string) (*TopicFilter, error) {
	if value == "" {
		return nil, fmt.Errorf("topic filter: %s cannot be empty", value)
	}

	if len(value) > 65535 {
		return nil, fmt.Errorf("topic filter: %s cannot be have more than 65535 bytes", value)
	}

	if !topicFilterRegex.MatchString(value) {
		return nil, fmt.Errorf("topic filter: %s format is invalid", value)
	}

	return &TopicFilter{value}, nil
}

func (t *TopicFilter) Match(topic_name *TopicName) bool {
	var tnItr = strings.Split(topic_name.Value, "/")
	var ftItr = strings.Split(t.Value, "/")

	var firstFt = ftItr[0]
	var firstTn = tnItr[0]

	tnItr = tnItr[1:]
	ftItr = ftItr[1:]

	if strings.HasPrefix(firstTn, "$") {
		if firstTn != firstFt {
			return false
		}
	} else {
		switch firstFt {
		case "#":
			return true
		case "+":
			break
		default:
			if firstTn != firstFt {
				return false
			}
		}
	}

loop:
	for {
		if len(tnItr) == 0 && len(ftItr) == 0 {
			break
		}

		if len(tnItr) == 0 {
			if ftItr[0] != "#" {
				return false
			} else {
				break
			}
		}

		if len(ftItr) == 0 {
			return false
		}

		var ft = ftItr[0]
		var tn = tnItr[0]

		tnItr = tnItr[1:]
		ftItr = ftItr[1:]

		switch ft {
		case "#":
			break loop
		case "+":
			continue
		default:
			if ft != tn {
				return false
			}
		}
	}

	return true
}
