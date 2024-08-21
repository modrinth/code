import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'
import loadCssMixin from './mixins/macCssFix.js'
import { createPlugin } from '@vintl/vintl/plugin'

import './../src/assets/stylesheets/tailwind.css'

const VIntlPlugin = createPlugin({
  controllerOpts: {
    defaultLocale: 'en-US',
    locale: 'en-US',
    locales: [
      {
        tag: 'en-US',
        meta: {
          displayName: 'American English',
        },
      },
    ],
  },
  globalMixin: true,
  injectInto: [],
})

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)
app.use(VIntlPlugin)

app.mount('#app')
