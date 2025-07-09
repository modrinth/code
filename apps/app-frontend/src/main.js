import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'
import { createPlugin } from '@vintl/vintl/plugin'
import * as Sentry from '@sentry/vue'
import { VueScanPlugin } from '@taijased/vue-render-tracker'

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

const vueScan = new VueScanPlugin({
  enabled: false, // Enable or disable the tracker
  showOverlay: true, // Show overlay to visualize renders
  log: false, // Log render events to the console
  playSound: false, // Play sound on each render
})

const pinia = createPinia()

let app = createApp(App)

Sentry.init({
  app,
  dsn: 'https://9508775ee5034536bc70433f5f531dd4@o485889.ingest.us.sentry.io/4504579615227904',
  integrations: [Sentry.browserTracingIntegration({ router })],
  tracesSampleRate: 0.1,
})

app.use(vueScan)
app.use(router)
app.use(pinia)
app.use(FloatingVue, {
  themes: {
    'ribbit-popout': {
      $extend: 'dropdown',
      placement: 'bottom-end',
      instantMove: true,
      distance: 8,
    },
  },
})
app.use(VIntlPlugin)

app.mount('#app')
