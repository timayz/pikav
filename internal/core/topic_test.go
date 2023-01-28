package core_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/timada-org/pikav/internal/core"
)

func TestFilterValidate(t *testing.T) {

	t.Run("#", func(t *testing.T) {
		_, err := core.NewFilter("#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis/player1")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/ranking", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis/player1/ranking")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/player1/#", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis/player1/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis/#", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport/tennis#", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis#")
		require.Error(t, err)
	})

	t.Run("sport/tennis/#/ranking", func(t *testing.T) {
		_, err := core.NewFilter("sport/tennis/#/ranking")
		require.Error(t, err)
	})

	t.Run("+", func(t *testing.T) {
		_, err := core.NewFilter("+")
		require.NoError(t, err)
	})

	t.Run("+/tennis/#", func(t *testing.T) {
		_, err := core.NewFilter("+/tennis/#")
		require.NoError(t, err)
	})

	t.Run("sport+", func(t *testing.T) {
		_, err := core.NewFilter("sport+")
		require.Error(t, err)
	})

	t.Run("sport/+/player1", func(t *testing.T) {
		_, err := core.NewFilter("sport/+/player1")
		require.NoError(t, err)
	})

	t.Run("+/+", func(t *testing.T) {
		_, err := core.NewFilter("+/+")
		require.NoError(t, err)
	})

	t.Run("$SYS/#", func(t *testing.T) {
		_, err := core.NewFilter("$SYS/#")
		require.NoError(t, err)
	})

	t.Run("$SYS", func(t *testing.T) {
		_, err := core.NewFilter("$SYS")
		require.NoError(t, err)
	})
}

func TestFilterMatch(t *testing.T) {
	t.Run("sport/#", func(t *testing.T) {
		filter, err := core.NewFilter("sport/#")
		require.NoError(t, err)

		name, _ := core.NewName("sport")
		assert.True(t, filter.Match(name))
	})

	t.Run("#", func(t *testing.T) {
		filter, err := core.NewFilter("#")
		require.NoError(t, err)

		n1, _ := core.NewName("sport")
		assert.True(t, filter.Match(n1))
		n2, _ := core.NewName("/")
		assert.True(t, filter.Match(n2))
		n3, _ := core.NewName("abc/def")
		assert.True(t, filter.Match(n3))
		n4, _ := core.NewName("$SYS")
		assert.False(t, filter.Match(n4))
		n5, _ := core.NewName("$SYS/abc")
		assert.False(t, filter.Match(n5))
	})

	t.Run("+/monitor/Clients", func(t *testing.T) {
		filter, err := core.NewFilter("+/monitor/Clients")
		require.NoError(t, err)

		n, _ := core.NewName("$SYS/monitor/Clients")
		assert.False(t, filter.Match(n))
	})

	t.Run("$SYS/#", func(t *testing.T) {
		filter, err := core.NewFilter("$SYS/#")
		require.NoError(t, err)

		n1, _ := core.NewName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n1))
		n2, _ := core.NewName("$SYS")
		assert.True(t, filter.Match(n2))
	})

	t.Run("$SYS/monitor/+", func(t *testing.T) {
		filter, err := core.NewFilter("$SYS/monitor/+")
		require.NoError(t, err)

		n, _ := core.NewName("$SYS/monitor/Clients")
		assert.True(t, filter.Match(n))
	})
}

func TestNameValidate(t *testing.T) {

	t.Run("sys", func(t *testing.T) {
		_, err1 := core.NewName("$SYS")
		require.NoError(t, err1)
		_, err2 := core.NewName("$SYS/broker/connection/test.cosm-energy/state")
		require.NoError(t, err2)
	})

	t.Run("slash", func(t *testing.T) {
		_, err := core.NewName("/")
		require.NoError(t, err)
	})

	t.Run("basic", func(t *testing.T) {
		_, err1 := core.NewName("/finance")
		require.NoError(t, err1)
		_, err2 := core.NewName("/finance//def")
		require.NoError(t, err2)
	})
}
