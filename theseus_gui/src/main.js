import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import 'omorphia/dist/style.css'
import '@/assets/stylesheets/global.scss'
import 'floating-vue/dist/style.css'
import FloatingVue from 'floating-vue'
import { initialize_state } from '@/helpers/state'
import loadCssMixin from './mixins/macCssFix.js'

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)

// FIXME: this causes the app to lag
// import * as Sentry from '@sentry/vue'
//
// Sentry.init({
//   app,
//   dsn: 'https://19a14416dafc4b4a858fa1a38db3b704@o485889.ingest.sentry.io/4505349067374592',
//   integrations: [],
//   enableTracing: false,
// })

const mountedApp = app.mount('#app')

initialize_state()
  .then(() => mountedApp.initialize())
  .catch((err) => {
    console.error(err)
  })
