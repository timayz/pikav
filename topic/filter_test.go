package topic_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/timada-org/pikav/topic"
)

func TestFilterValidate(t *testing.T) {

	t.Run("#", func(t *testing.T) {
		_, err := topic.NewFilter("#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis/player1")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/ranking", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis/player1/ranking")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/#", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis/player1/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/#", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis#", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis#")
		require.Error(t, err)
	})

	t.Run("sport/tennis/#/ranking", func(t *testing.T) {
		_, err := topic.NewFilter("sport/tennis/#/ranking")
		require.Error(t, err)
	})

	t.Run("+", func(t *testing.T) {
		_, err := topic.NewFilter("+")
		require.NoError(t, err)
	})

	t.Run("+/tennis/#", func(t *testing.T) {
		_, err := topic.NewFilter("+/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport+", func(t *testing.T) {
		_, err := topic.NewFilter("sport+")
		require.Error(t, err)
	})

	t.Run("sport/+/player1", func(t *testing.T) {
		_, err := topic.NewFilter("sport/+/player1")
		require.NoError(t, err)
	})

	t.Run("+/+", func(t *testing.T) {
		_, err := topic.NewFilter("+/+")
		require.NoError(t, err)
	})

	t.Run("$SYS/#", func(t *testing.T) {
		_, err := topic.NewFilter("$SYS/#")
		require.NoError(t, err)
	})

	t.Run("$SYS", func(t *testing.T) {
		_, err := topic.NewFilter("$SYS")
		require.NoError(t, err)
	})
}

func TestFilterMatch(t *testing.T) {
	t.Run("sport/#", func(t *testing.T) {
		filter, err := topic.NewFilter("sport/#")
		require.NoError(t, err)

		name, _ := topic.NewName("sport")
		assert.True(t, filter.Match(name))
	})

	t.Run("#", func(t *testing.T) {
		filter, err := topic.NewFilter("#")
		require.NoError(t, err)

		n1, _ := topic.NewName("sport")
		assert.True(t, filter.Match(n1))
		n2, _ := topic.NewName("/")
		assert.True(t, filter.Match(n2))
		n3, _ := topic.NewName("abc/def")
		assert.True(t, filter.Match(n3))
		n4, _ := topic.NewName("$SYS")
		assert.False(t, filter.Match(n4))
		n5, _ := topic.NewName("$SYS/abc")
		assert.False(t, filter.Match(n5))
	})

	t.Run("+/monitor/Clients", func(t *testing.T) {
		filter, err := topic.NewFilter("+/monitor/Clients")
		require.NoError(t, err)

		n, _ := topic.NewName("$SYS/monitor/Clients")
		assert.False(t, filter.Match(n))
	})

	t.Run("$SYS/#", func(t *testing.T) {
		filter, err := topic.NewFilter("$SYS/#")
		require.NoError(t, err)

		n1, _ := topic.NewName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n1))
		n2, _ := topic.NewName("$SYS")
		assert.True(t, filter.Match(n2))
	})

	t.Run("$SYS/monitor/+", func(t *testing.T) {
		filter, err := topic.NewFilter("$SYS/monitor/+")
		require.NoError(t, err)

		n, _ := topic.NewName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n))
	})
}
