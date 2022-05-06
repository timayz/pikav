package topic_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/timada-org/pikav/topic"
)

func TestFilterValidate(t *testing.T) {

	t.Run("#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis/player1")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/ranking", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis/player1/ranking")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis/player1/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis#")
		require.Error(t, err)
	})

	t.Run("sport/tennis/#/ranking", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/tennis/#/ranking")
		require.Error(t, err)
	})

	t.Run("+", func(t *testing.T) {
		_, err := topic.NewTopicFilter("+")
		require.NoError(t, err)
	})

	t.Run("+/tennis/#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("+/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport+", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport+")
		require.Error(t, err)
	})

	t.Run("sport/+/player1", func(t *testing.T) {
		_, err := topic.NewTopicFilter("sport/+/player1")
		require.NoError(t, err)
	})

	t.Run("+/+", func(t *testing.T) {
		_, err := topic.NewTopicFilter("+/+")
		require.NoError(t, err)
	})

	t.Run("$SYS/#", func(t *testing.T) {
		_, err := topic.NewTopicFilter("$SYS/#")
		require.NoError(t, err)
	})

	t.Run("$SYS", func(t *testing.T) {
		_, err := topic.NewTopicFilter("$SYS")
		require.NoError(t, err)
	})
}

func TestFilterMatch(t *testing.T) {
	t.Run("sport/#", func(t *testing.T) {
		filter, err := topic.NewTopicFilter("sport/#")
		require.NoError(t, err)

		name, _ := topic.NewTopicName("sport")
		assert.True(t, filter.Match(name))
	})

	t.Run("#", func(t *testing.T) {
		filter, err := topic.NewTopicFilter("#")
		require.NoError(t, err)

		n1, _ := topic.NewTopicName("sport")
		assert.True(t, filter.Match(n1))
		n2, _ := topic.NewTopicName("/")
		assert.True(t, filter.Match(n2))
		n3, _ := topic.NewTopicName("abc/def")
		assert.True(t, filter.Match(n3))
		n4, _ := topic.NewTopicName("$SYS")
		assert.False(t, filter.Match(n4))
		n5, _ := topic.NewTopicName("$SYS/abc")
		assert.False(t, filter.Match(n5))
	})

	t.Run("+/monitor/Clients", func(t *testing.T) {
		filter, err := topic.NewTopicFilter("+/monitor/Clients")
		require.NoError(t, err)

		n, _ := topic.NewTopicName("$SYS/monitor/Clients")
		assert.False(t, filter.Match(n))
	})

	t.Run("$SYS/#", func(t *testing.T) {
		filter, err := topic.NewTopicFilter("$SYS/#")
		require.NoError(t, err)

		n1, _ := topic.NewTopicName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n1))
		n2, _ := topic.NewTopicName("$SYS")
		assert.True(t, filter.Match(n2))
	})

	t.Run("$SYS/monitor/+", func(t *testing.T) {
		filter, err := topic.NewTopicFilter("$SYS/monitor/+")
		require.NoError(t, err)

		n, _ := topic.NewTopicName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n))
	})
}
