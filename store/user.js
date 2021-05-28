export const state = () => ({
  notifications: {
    count: 0,
    lastUpdated: 0,
  },
})

export const mutations = {
  SET_NOTIFICATIONS(state, count) {
    state.notifications.count = count
    state.notifications.lastUpdated = Date.now()
  },
}

export const actions = {
  async fetchNotifications(
    { commit, state, rootState },
    { force = false } = {}
  ) {
    if (
      rootState.auth.user &&
      rootState.auth.user.id &&
      (force || Date.now() - state.notifications.lastUpdated > 300000)
    ) {
      const notifications = (
        await this.$axios.get(
          `user/${rootState.auth.user.id}/notifications`,
          rootState.auth.headers
        )
      ).data

      commit('SET_NOTIFICATIONS', notifications.length)
    }
  },
}
