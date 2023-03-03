import { createRouter, createWebHistory } from 'vue-router'

import Index from './pages/Index.vue'
import About from './pages/About.vue'

export default new createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'index',
      component: Index,
    },
    {
      path: '/about',
      name: 'about',
      component: About,
    },
  ],
})
