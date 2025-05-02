import { createRouter, createWebHistory } from 'vue-router'
import * as Pages from '@/pages'
import * as Project from '@/pages/project'
import * as Instance from '@/pages/instance'
import * as Library from '@/pages/library'

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
      meta: {
        breadcrumb: [{ name: 'Home' }],
      },
    },
    {
      path: '/worlds',
      name: 'Worlds',
      component: Pages.Worlds,
      meta: {
        breadcrumb: [{ name: 'Worlds' }],
      },
    },
    {
      path: '/browse/:projectType',
      name: 'Discover content',
      component: Pages.Browse,
      meta: {
        breadcrumb: [{ name: 'Discover content' }],
      },
    },
    {
      path: '/library',
      name: 'Library',
      component: Library.Index,
      meta: {
        breadcrumb: [{ name: 'Library' }],
      },
      children: [
        {
          path: '',
          name: 'Overview',
          component: Library.Overview,
        },
        {
          path: 'downloaded',
          name: 'Downloaded',
          component: Library.Downloaded,
        },
        {
          path: 'custom',
          name: 'Custom',
          component: Library.Custom,
        },
      ],
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
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project' }],
          },
        },
        {
          path: 'versions',
          name: 'Versions',
          component: Project.Versions,
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Versions' }],
          },
        },
        {
          path: 'version/:version',
          name: 'Version',
          component: Project.Version,
          props: true,
          meta: {
            useContext: true,
            breadcrumb: [
              { name: '?Project', link: '/project/{id}/' },
              { name: 'Versions', link: '/project/{id}/versions' },
              { name: '?Version' },
            ],
          },
        },
        {
          path: 'gallery',
          name: 'Gallery',
          component: Project.Gallery,
          meta: {
            useContext: true,
            breadcrumb: [{ name: '?Project', link: '/project/{id}/' }, { name: 'Gallery' }],
          },
        },
      ],
    },
    {
      path: '/instance/:id',
      name: 'Instance',
      component: Instance.Index,
      props: true,
      children: [
        // {
        //   path: '',
        //   name: 'Overview',
        //   component: Instance.Overview,
        //   meta: {
        //     useRootContext: true,
        //     breadcrumb: [{ name: '?Instance' }],
        //   },
        // },
        {
          path: 'worlds',
          name: 'InstanceWorlds',
          component: Instance.Worlds,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Worlds' }],
          },
        },
        {
          path: '',
          name: 'Mods',
          component: Instance.Mods,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Content' }],
          },
        },
        {
          path: 'projects/:type',
          name: 'ModsFilter',
          component: Instance.Mods,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Content' }],
          },
        },
        {
          path: 'logs',
          name: 'Logs',
          component: Instance.Logs,
          meta: {
            useRootContext: true,
            breadcrumb: [{ name: '?Instance', link: '/instance/{id}/' }, { name: 'Logs' }],
          },
        },
      ],
    },
  ],
  linkActiveClass: 'router-link-active',
  linkExactActiveClass: 'router-link-exact-active',
  scrollBehavior() {
    // Sometimes Vue's scroll behavior is not working as expected, so we need to manually scroll to top (especially on Linux)
    document.querySelector('.app-viewport')?.scrollTo(0, 0)
    return {
      el: '.app-viewport',
      top: 0,
    }
  },
})
