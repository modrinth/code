import { createRouter, createWebHistory } from 'vue-router'

import * as Pages from '@/pages'
import * as Hosting from '@/pages/hosting/manage'
import * as Instance from '@/pages/instance'
import * as Library from '@/pages/library'
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
			path: '/hosting/manage/',
			name: 'Servers',
			component: Pages.Servers,
		},
		{
			path: '/hosting/manage/:id',
			name: 'ServerManage',
			component: Hosting.Index,
			children: [
				{
					path: '',
					name: 'ServerManageOverview',
					component: Hosting.Overview,
				},
				{
					path: 'content',
					name: 'ServerManageContent',
					component: Hosting.Content,
				},
				{
					path: 'files',
					name: 'ServerManageFiles',
					component: Hosting.Files,
				},
				{
					path: 'backups',
					name: 'ServerManageBackups',
					component: Hosting.Backups,
				},
				{
					path: 'access',
					name: 'ServerManageAccess',
					component: Hosting.Access,
				},
			],
		},
		{
			path: '/browse/:projectType',
			name: 'Discover content',
			component: Pages.Browse,
		},
		{
			path: '/skins',
			name: 'Skin selector',
			component: Pages.Skins,
		},
		{
			path: '/user/:user/:projectType?',
			name: 'User',
			component: Pages.User,
		},
		{
			path: '/library',
			name: 'Library',
			component: Library.Index,
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
					path: 'modpacks',
					name: 'Modpacks',
					component: Library.Modpacks,
				},
				{
					path: 'servers',
					name: 'LibraryServers',
					component: Library.Servers,
				},
				{
					path: 'custom',
					name: 'Custom',
					component: Library.Custom,
				},
			],
		},
		{
			path: '/:projectType(mod|plugin|datapack|resourcepack|shader|modpack)/:id/:rest(.*)*',
			redirect: (to) => {
				const rest = to.params.rest ? `/${[].concat(to.params.rest).join('/')}` : ''
				return `/project/${to.params.id}${rest}${to.hash}`
			},
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
					path: 'version/:version',
					name: 'Version',
					component: Project.Version,
					props: true,
				},
				{
					path: 'gallery',
					name: 'Gallery',
					component: Project.Gallery,
				},
			],
		},
		{
			path: '/instance/:id',
			name: 'Instance',
			component: Instance.Index,
			props: true,
			children: [
				{
					path: 'worlds',
					name: 'InstanceWorlds',
					component: Instance.Worlds,
				},
				{
					path: 'share',
					name: 'InstanceShare',
					component: Instance.Share,
				},
				{
					path: '',
					name: 'Mods',
					component: Instance.Mods,
				},
				{
					path: 'projects/:type',
					name: 'ModsFilter',
					component: Instance.Mods,
				},
				{
					path: 'files',
					name: 'Files',
					component: Instance.Files,
				},
				{
					path: 'logs',
					name: 'Logs',
					component: Instance.Logs,
					meta: {
						renderMode: 'fixed',
					},
				},
			],
		},
	],
	linkActiveClass: 'router-link-active',
	linkExactActiveClass: 'router-link-exact-active',
	scrollBehavior(to, from) {
		if (to.path === from.path) return
		// Sometimes Vue's scroll behavior is not working as expected, so we need to manually scroll to top (especially on Linux)
		document.querySelector('.app-viewport')?.scrollTo(0, 0)
		return {
			el: '.app-viewport',
			top: 0,
		}
	},
})
