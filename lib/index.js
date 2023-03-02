import * as components from './components'

function install(app) {
  for (const key in components) {
    app.component(key, components[key])
  }
}

export default { install }
export * from './components'

import './assets/omorphia.scss'
