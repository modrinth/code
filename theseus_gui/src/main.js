import { createApp } from 'vue'
import store from './store/state'
import router from './routes'
import App from './App.vue'
import '../node_modules/omorphia/dist/style.css'
import './style.css'

createApp(App).use(router).use(store).mount('#app')
