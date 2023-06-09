import { createApp } from 'vue'
import router from '@/routes.js'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import 'omorphia/dist/style.css'
import '@/assets/stylesheets/global.scss'
import 'floating-vue/dist/style.css'
import FloatingVue from 'floating-vue'
import { initialize_state } from '@/helpers/state'
import loadCssMixin from './mixins/macCssFix'
import { createWebTauri } from './web'

// @ts-ignore
if (import.meta.env.TAURI_WEB_DEV) {
  createWebTauri();
}

const pinia = createPinia()

const app = createApp(App)

app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)

const mountedApp = app.mount('#app')

initialize_state()
  .then(() => mountedApp.initialize())
  .catch((err) => {
    console.error(err)
  })
