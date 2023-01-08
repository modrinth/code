export const state = () => ({
  notifications: [],
  follows: [],
  projects: [],
  lastUpdated: 0,
})

export const mutations = {
  SET_NOTIFICATIONS(state, notifications) {
    state.notifications = notifications
  },
  SET_FOLLOWS(state, follows) {
    state.follows = follows
  },
  SET_PROJECTS(state, projects) {
    state.projects = projects
  },
  SET_LAST_UPDATED(state, lastUpdated) {
    state.lastUpdated = lastUpdated
  },
}

export const actions = {
  async fetchAll({ commit, state, rootState }, { force = false } = {}) {
    if (
      rootState.auth.user &&
      rootState.auth.user.id &&
      (force || Date.now() - state.lastUpdated > 300000)
    ) {
      try {
        const [notifications, follows, projects] = (
          await Promise.all([
            this.$axios.get(
              `user/${rootState.auth.user.id}/notifications`,
              rootState.auth.headers
            ),
            this.$axios.get(
              `user/${rootState.auth.user.id}/follows`,
              rootState.auth.headers
            ),
            this.$axios.get(
              `user/${rootState.auth.user.id}/projects`,
              rootState.auth.headers
            ),
          ])
        ).map((it) => it.data)

        commit('SET_NOTIFICATIONS', notifications)
        commit('SET_FOLLOWS', follows)
        commit('SET_PROJECTS', projects)
        commit('SET_LAST_UPDATED', Date.now())
      } catch (err) {
        console.error(err)
      }
    }
  },
  async fetchNotifications({ commit, rootState }) {
    if (rootState.auth.user && rootState.auth.user.id) {
      try {
        const notifications = (
          await this.$axios.get(
            `user/${rootState.auth.user.id}/notifications`,
            rootState.auth.headers
          )
        ).data

        commit('SET_NOTIFICATIONS', notifications)
      } catch (err) {
        console.error(err)
      }
    }
  },
  async fetchFollows({ commit, rootState }) {
    if (rootState.auth.user && rootState.auth.user.id) {
      try {
        const follows = (
          await this.$axios.get(
            `user/${rootState.auth.user.id}/follows`,
            rootState.auth.headers
          )
        ).data

        commit('SET_FOLLOWS', follows)
      } catch (err) {
        console.error(err)
      }
    }
  },
  async fetchProjects({ commit, rootState }) {
    if (rootState.auth.user && rootState.auth.user.id) {
      try {
        const projects = (
          await this.$axios.get(
            `user/${rootState.auth.user.id}/projects`,
            rootState.auth.headers
          )
        ).data

        commit('SET_PROJECTS', projects)
      } catch (err) {
        console.error(err)
      }
    }
  },
  followProject({ commit, state, rootState }, project) {
    commit('SET_FOLLOWS', state.follows.concat(project))

    setTimeout(() => {
      this.$axios.post(
        `project/${project.id}/follow`,
        {},
        rootState.auth.headers
      )
    })
  },
  unfollowProject({ commit, state, rootState }, project) {
    commit(
      'SET_FOLLOWS',
      state.follows.filter((x) => x.id !== project.id)
    )

    setTimeout(() => {
      this.$axios.delete(`project/${project.id}/follow`, rootState.auth.headers)
    })
  },
  deleteNotification({ commit, state, rootState }, id) {
    commit(
      'SET_NOTIFICATIONS',
      state.notifications.filter((x) => x.id !== id)
    )
  },
}
