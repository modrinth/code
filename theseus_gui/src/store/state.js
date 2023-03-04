import { createStore } from 'vuex'

export default createStore({
  state() {
    return {
      count: 0,
      darkTheme: true,
    }
  },
  mutations: {
    increment(state) {
      state.count++
    },
    decrement(state) {
      state.count--
    },
    toggleTheme(state) {
      state.darkTheme = !state.darkTheme
    },
  },
})
