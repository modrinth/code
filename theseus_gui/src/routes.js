import { createRouter, createWebHistory } from 'vue-router'
import { Index, Browse, Library, AddInstance, Project } from './pages'

export default new createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'index',
      component: Index,
    },
    {
      path: '/browse',
      name: 'browse',
      component: Browse,
    },
    {
      path: '/library',
      name: 'library',
      component: Library,
    },
    {
      path: '/add-instance',
      name: 'add-instance',
      component: AddInstance,
    },
    {
      path: '/project',
      name: 'project',
      component: Project,
    },
  ],
})
