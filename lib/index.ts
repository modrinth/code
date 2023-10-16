import * as components from './components/index.js'
import FloatingVue from 'floating-vue'
import { Plugin } from 'vue'

const plugin: Plugin = (app) => {
  for (const key in components) {
    app.component(key, components[key as keyof typeof components])
  }
  app.use(FloatingVue)
}

export default plugin
export * from './components/index.js'
export * from './helpers/index.js'

import './assets/omorphia.scss'
