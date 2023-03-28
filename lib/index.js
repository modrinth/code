import * as components from './components'
import FloatingVue from 'floating-vue'

function install(app) {
  for (const key in components) {
    app.component(key, components[key])
  }
  app.use(FloatingVue)
}

export default { install }
export * from './components'
export * from './helpers'

import './assets/omorphia.scss'
