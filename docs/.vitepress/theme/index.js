import DefaultTheme from 'vitepress/theme'
import Omorphia from 'omorphia'
import DemoContainer from './DemoContainer.vue'

import './compat.scss'
import './style.scss'

export default {
  ...DefaultTheme,
  enhanceApp(ctx) {
    ctx.app.use(Omorphia)
    ctx.app.component('DemoContainer', DemoContainer)
  },
}
