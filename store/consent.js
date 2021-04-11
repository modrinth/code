const VERSION = 1
const parameters = {
  maxAge: 60 * 60 * 24 * 365 * 10, // Ten years
  sameSite: 'Strict',
  secure: true,
  httpOnly: false,
  path: '/',
}

export const state = () => ({
  is_consent_given: false,
  scopes_allowed: [],
  loaded: false,
})
export const mutations = {
  loaded(state) {
    state.loaded = true
  },
  set_consent(state, val) {
    state.is_consent_given = val
  },
  add_scope(state, val) {
    // Check if the scope is not already provided
    if (state.scopes_allowed.includes(val)) return
    state.scopes_allowed.push(val)
  },
  remove_scope(state, val) {
    const pos = state.scopes_allowed.findIndex((el) => el === val)
    if (pos >= 0) state.scopes_allowed.splice(pos, 1)
  },
}
export const actions = {
  loadFromCookies(state, $cookies) {
    if (state.state.loaded) {
      return
    }
    state.commit('set_consent', $cookies.get('modrinth-consent') === true)
    const scopes = $cookies.get('modrinth-scopes')
    if (!scopes) return
    scopes.split(',').forEach((elem) => {
      state.commit('add_scope', elem)
    })
    state.commit('loaded')
  },
  save(state, $cookies) {
    $cookies.set('modrinth-consent', state.state.is_consent_given, parameters)
    $cookies.set('modrinth-version', VERSION, parameters)
    $cookies.set(
      'modrinth-scopes',
      state.state.scopes_allowed.join(','),
      parameters
    )
  },
}
export const getters = {
  is_scope_allowed: (state) => (id) => {
    return state.scopes_allowed.contains(id)
  },
}
