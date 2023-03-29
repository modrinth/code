import { createRouter, createWebHistory } from 'vue-router'
import * as Pages from '@/pages'
import * as Project from '@/pages/project'

/**
 * Configures application routing. Add page to pages/index and then add to route table here.
 */
export default new createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Home',
      component: Pages.Index,
    },
    {
      path: '/browse',
      name: 'Browse',
      component: Pages.Browse,
    },
    {
      path: '/library',
      name: 'Library',
      component: Pages.Library,
    },
    {
      path: '/project',
      name: 'Project',
      component: Pages.Project,
    },
    {
      path: '/settings',
      name: 'Settings',
      component: Pages.Settings,
    },
    {
      path: '/project/:id',
      name: 'Project',
      component: Project.Index,
      props: true,
      children: [
        {
          path: '',
          name: 'Description',
          component: Project.Description,
        },
        {
          path: 'versions',
          name: 'Versions',
          component: Project.Versions,
        },
        {
          path: 'gallery',
          name: 'Gallery',
          component: Project.Gallery,
        }
      ]
    }
  ],
  linkActiveClass: 'router-link-active',
  linkExactActiveClass: 'router-link-exact-active',
})
