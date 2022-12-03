const parameters = {
  maxAge: 60 * 60 * 24 * 365 * 10, // Ten years
  sameSite: 'Strict',
  secure: true,
  httpOnly: false,
  path: '/',
}

export const defaults = {
  searchLayout: false,
  projectLayout: false,
  modpacksAlphaNotice: true,
  advancedRendering: true,
  externalLinksNewTab: true,
  notUsingBlockers: false,
  searchDisplayMode: {
    mod: 'list',
    plugin: 'list',
    resourcepack: 'gallery',
    modpack: 'list',
    user: 'list',
  },
}

export const state = () => defaults

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
  SET_EXTERNAL_LINKS_NEW_TAB(state, externalLinksNewTab) {
    state.externalLinksNewTab = externalLinksNewTab
  },
  SET_SEARCH_DISPLAY_MODE(state, { projectType, mode }) {
    state.searchDisplayMode[projectType] = mode
  },
  SET_NOT_USING_BLOCKERS(state, notUsingBlockers) {
    state.notUsingBlockers = notUsingBlockers
  },
}

export const actions = {
  fetchCosmetics({ commit }, $cookies) {
    commit(
      'SET_PROJECT_LAYOUT',
      $cookies.get('project-layout') ?? defaults.projectLayout
    )
    commit(
      'SET_SEARCH_LAYOUT',
      $cookies.get('search-layout') ?? defaults.searchLayout
    )
    commit(
      'SET_MODPACKS_ALPHA_NOTICE',
      $cookies.get('modpacks-alpha-notice') ?? defaults.modpacksAlphaNotice
    )
    commit(
      'SET_ADVANCED_RENDERING',
      $cookies.get('advanced-rendering') ?? defaults.advancedRendering
    )
    commit(
      'SET_EXTERNAL_LINKS_NEW_TAB',
      $cookies.get('external-links-new-tab') ?? defaults.externalLinksNewTab
    )
    Object.keys(defaults.searchDisplayMode).forEach((projectType) => {
      commit('SET_SEARCH_DISPLAY_MODE', {
        projectType,
        mode:
          $cookies.get('search-display-mode-' + projectType) ??
          defaults.searchDisplayMode[projectType],
      })
    })
  },
  save(
    { commit },
    {
      projectLayout,
      searchLayout,
      modpacksAlphaNotice,
      advancedRendering,
      externalLinksNewTab,
      $cookies,
    }
  ) {
    commit('SET_PROJECT_LAYOUT', projectLayout)
    commit('SET_SEARCH_LAYOUT', searchLayout)
    commit('SET_MODPACKS_ALPHA_NOTICE', modpacksAlphaNotice)
    commit('SET_ADVANCED_RENDERING', advancedRendering)
    commit('SET_EXTERNAL_LINKS_NEW_TAB', externalLinksNewTab)

    $cookies.set('project-layout', projectLayout, parameters)
    $cookies.set('search-layout', searchLayout, parameters)
    $cookies.set('modpacks-alpha-notice', modpacksAlphaNotice, parameters)
    $cookies.set('advanced-rendering', advancedRendering, parameters)
    $cookies.set('external-links-new-tab', externalLinksNewTab, parameters)
  },
  saveSearchDisplayMode({ commit }, { projectType, mode, $cookies }) {
    commit('SET_SEARCH_DISPLAY_MODE', { projectType, mode })

    $cookies.set('search-display-mode-' + projectType, mode, parameters)
  },
}
