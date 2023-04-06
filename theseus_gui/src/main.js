import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import '../node_modules/omorphia/dist/style.css'
import '@/assets/stylesheets/global.css'
import FloatingVue from 'floating-vue'
import { initialize_state } from '@/helpers/state'

const pinia = createPinia()

await initialize_state();

createApp(App).use(router).use(pinia).use(FloatingVue).mount('#app')
