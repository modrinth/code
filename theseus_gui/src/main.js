import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import '../node_modules/omorphia/dist/style.css'
import '@/assets/stylesheets/global.css'

const pinia = createPinia()

createApp(App).use(router).use(pinia).mount('#app')
