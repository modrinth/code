import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import '../node_modules/omorphia/dist/style.css'
import '@/assets/stylesheets/global.scss'
import FloatingVue from 'floating-vue'
import { initialize_state } from '@/helpers/state'

const pinia = createPinia()

initialize_state().then(() => {
  createApp(App).use(router).use(pinia).use(FloatingVue).mount('#app')
})
