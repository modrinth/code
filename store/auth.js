export const state = () => ({
  user: null,
  userFollows: [],
  token: '',
  headers: {},
})

export const mutations = {
  SET_USER(state, user) {
    state.user = user
  },
  SET_USER_FOLLOWS(state, follows) {
    state.userFollows = follows
  },
  SET_TOKEN(state, token) {
    state.token = token
  },
  SET_HEADERS(state, headers) {
    state.headers = headers
  },
}

export const actions = {
  async fetchUser({ commit }, { token }) {
    try {
      const user = (
        await this.$axios.get(`https://api.modrinth.com/api/v1/user`, {
          headers: {
            Authorization: token,
          },
        })
      ).data

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
  async fetchUserFollows({ commit }, { userId, token }) {
    const follows = await this.$axios.get(
      `https://api.modrinth.com/api/v1/user/${userId}/follows`,
      {
        headers: {
          Authorization: token,
        },
      }
    )
    commit('SET_USER_FOLLOWS', follows)
  },
}
