import { createRouter, createWebHistory } from 'vue-router'
import * as Pages from '@/pages'
import * as Instance from '@/pages/instance'

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
      path: '/add-instance',
      name: 'Add Instance',
      component: Pages.AddInstance,
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
      path: '/instance/:id',
      name: 'Instance',
      component: Instance.Index,
      props: true,
      children: [
        {
          path: '',
          name: 'Mods',
          component: Instance.Mods,
        },
        {
          path: 'options',
          name: 'Options',
          component: Instance.Options,
        },
        {
          path: 'logs',
          name: 'Logs',
          component: Instance.Logs,
        },
      ],
    },
  ],
  linkActiveClass: 'router-link-active',
  linkExactActiveClass: 'router-link-exact-active',
})
