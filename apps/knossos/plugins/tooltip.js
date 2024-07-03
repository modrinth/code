import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'

export default defineNuxtPlugin(({ vueApp }) => {
  vueApp.use(FloatingVue)
})
