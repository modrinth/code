import DefaultLayout from '~/layouts/Default.vue'
import '~/assets/styles/global.scss'

export default function (Vue, { router, head, isClient }) {
  Vue.component('Layout', DefaultLayout)
}
