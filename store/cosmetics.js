const parameters = {
  maxAge: 60 * 60 * 24 * 365 * 10, // Ten years
  sameSite: 'Strict',
  secure: true,
  httpOnly: false,
  path: '/',
}

export const state = () => ({
  searchLayout: false,
  projectLayout: false,
  modpacksAlphaNotice: true,
  advancedRendering: true,
  notUsingBlockers: false,
})

export const mutations = {
  SET_SEARCH_LAYOUT(state, searchLayout) {
    state.searchLayout = searchLayout
  },
  SET_PROJECT_LAYOUT(state, projectLayout) {
    state.projectLayout = projectLayout
  },
  SET_MODPACKS_ALPHA_NOTICE(state, modpacksAlphaNotice) {
    state.modpacksAlphaNotice = modpacksAlphaNotice
  },
  SET_ADVANCED_RENDERING(state, advancedRendering) {
    state.advancedRendering = advancedRendering
  },
  SET_NOT_USING_BLOCKERS(state, notUsingBlockers) {
    state.notUsingBlockers = notUsingBlockers
  },
}

export const actions = {
  fetchCosmetics({ commit }, $cookies) {
    commit('SET_PROJECT_LAYOUT', $cookies.get('project-layout'))
    commit('SET_SEARCH_LAYOUT', $cookies.get('search-layout'))
    commit('SET_MODPACKS_ALPHA_NOTICE', $cookies.get('modpacks-alpha-notice'))
    commit('SET_ADVANCED_RENDERING', $cookies.get('advanced-rendering'))
  },
  save(
    { commit },
    {
      projectLayout,
      searchLayout,
      modpacksAlphaNotice,
      advancedRendering,
      $cookies,
    }
  ) {
    commit('SET_PROJECT_LAYOUT', projectLayout)
    commit('SET_SEARCH_LAYOUT', searchLayout)
    commit('SET_MODPACKS_ALPHA_NOTICE', modpacksAlphaNotice)
    commit('SET_ADVANCED_RENDERING', advancedRendering)

    $cookies.set('project-layout', projectLayout, parameters)
    $cookies.set('search-layout', searchLayout, parameters)
    $cookies.set('modpacks-alpha-notice', modpacksAlphaNotice, parameters)
    $cookies.set('advanced-rendering', advancedRendering, parameters)
  },
}
