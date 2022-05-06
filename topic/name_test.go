package topic_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/timada-org/pikav/topic"
)

func TestNameValidate(t *testing.T) {

	t.Run("sys", func(t *testing.T) {
		_, err1 := topic.NewTopicName("$SYS")
		require.NoError(t, err1)
		_, err2 := topic.NewTopicName("$SYS/broker/connection/test.cosm-energy/state")
		require.NoError(t, err2)
	})

	t.Run("slash", func(t *testing.T) {
		_, err := topic.NewTopicName("/")
		require.NoError(t, err)
	})

	t.Run("basic", func(t *testing.T) {
		_, err1 := topic.NewTopicName("/finance")
		require.NoError(t, err1)
		_, err2 := topic.NewTopicName("/finance//def")
		require.NoError(t, err2)
	})
}
