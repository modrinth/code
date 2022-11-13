export const state = () => ({
  user: null,
  token: '',
  headers: {},
})

export const mutations = {
  SET_USER(state, user) {
    state.user = user
  },
  SET_TOKEN(state, token) {
    state.token = token
  },
  SET_HEADERS(state, headers) {
    state.headers = headers
  },
}

export const actions = {
  async fetchUser({ commit, state }, { token }) {
    try {
      const user = (
        await this.$axios.get(`user`, {
          headers: {
            Authorization: token,
            'x-ratelimit-key': process.server
              ? process.env.RATE_LIMIT_IGNORE_KEY || ''
              : '',
          },
        })
      ).data

      if (user.payout_data && user.payout_data.balance) {
        user.payout_data.balance =
          Math.floor(user.payout_data.balance * 100) / 100
      }

      commit('SET_USER', user)
      commit('SET_TOKEN', token)
      commit('SET_HEADERS', {
        headers: {
          Authorization: token,
        },
      })
    } catch (e) {
      console.error('Request for user info encountered an error: ', e)
    }
  },
}
