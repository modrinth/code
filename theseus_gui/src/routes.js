import { createRouter, createWebHistory } from 'vue-router'
import { Index, Browse, Library, AddInstance, Project, Settings } from './pages'

export default new createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Home',
      component: Index,
    },
    {
      path: '/browse',
      name: 'Browse',
      component: Browse,
    },
    {
      path: '/library',
      name: 'Library',
      component: Library,
    },
    {
      path: '/add-instance',
      name: 'Add Instance',
      component: AddInstance,
    },
    {
      path: '/project',
      name: 'Project',
      component: Project,
    },
    {
      path: '/settings',
      name: 'Settings',
      component: Settings,
    },
  ],
})
