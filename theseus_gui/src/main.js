import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import 'omorphia/dist/style.css'
import '@/assets/stylesheets/global.scss'
import 'floating-vue/dist/style.css'
import FloatingVue from 'floating-vue'
import { get_opening_command, initialize_state } from '@/helpers/state'
import loadCssMixin from './mixins/macCssFix.js'
import { get } from '@/helpers/settings'

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)

const mountedApp = app.mount('#app')

initialize_state()
  .then(() => {
    // First, redirect to other landing page if we have that setting
    get()
      .then((fetchSettings) => {
        if (fetchSettings?.default_page && fetchSettings?.default_page !== 'Home') {
          router.push({ name: fetchSettings?.default_page })
        }
      })
      .catch((err) => {
        console.error(err)
      })
      .finally(() => {
        mountedApp.initialize()
        get_opening_command().then((command) => {
          console.log(JSON.stringify(command)) // change me to use whatever FE command handler is made
        })
      })
  })
  .catch((err) => {
    console.error(err)
  })
