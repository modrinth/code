import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import 'omorphia/dist/style.css'
import '@/assets/stylesheets/global.scss'
import FloatingVue from 'floating-vue'
import { initialize_state } from '@/helpers/state'
import loadCssMixin from './mixins/macCssFix.js'

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.mixin(loadCssMixin)

initialize_state()
  .then(() => app.mount('#app'))
  .catch((err) => {
    console.error(err)
  })
