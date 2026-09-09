import { createRouter, createWebHistory } from 'vue-router'
/**
 * Loads each page when its route is visited.
 */
export default new createRouter({
	history: createWebHistory(),
	routes: [
		{
			path: '/',
			name: 'Home',
			component: () => import('@/pages/Index.vue'),
		},
		{
			path: '/hosting/manage/',
			name: 'Servers',
			component: () => import('@/pages/Servers.vue'),
		},
		{
			path: '/hosting/manage/:id',
			name: 'ServerManage',
			component: () => import('@/pages/hosting/manage/Index.vue'),
			children: [
				{
					path: '',
					name: 'ServerManageOverview',
					component: () => import('@/pages/hosting/manage/Overview.vue'),
				},
				{
					path: 'content',
					name: 'ServerManageContent',
					component: () => import('@/pages/hosting/manage/Content.vue'),
				},
				{
					path: 'files',
					name: 'ServerManageFiles',
					component: () => import('@/pages/hosting/manage/Files.vue'),
				},
				{
					path: 'backups',
					name: 'ServerManageBackups',
					component: () => import('@/pages/hosting/manage/Backups.vue'),
				},
				{
					path: 'access',
					name: 'ServerManageAccess',
					component: () => import('@/pages/hosting/manage/Access.vue'),
				},
			],
		},
		{
			path: '/browse/:projectType',
			name: 'Discover content',
			component: () => import('@/pages/Browse.vue'),
		},
		{
			path: '/skins',
			name: 'Skin selector',
			component: () => import('@/pages/Skins.vue'),
		},
		{
			path: '/screenshots',
			name: 'Screenshots',
			component: () => import('@/pages/Screenshots.vue'),
		},
		{
			path: '/user/:user/:projectType?',
			name: 'User',
			component: () => import('@/pages/User.vue'),
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
			component: () => import('@/pages/project/Index.vue'),
			props: true,
			children: [
				{
					path: '',
					name: 'Description',
					component: () => import('@/pages/project/Description.vue'),
				},
				{
					path: 'versions',
					name: 'Versions',
					component: () => import('@/pages/project/Versions.vue'),
				},
				{
					path: 'version/:version',
					name: 'Version',
					component: () => import('@/pages/project/Version.vue'),
					props: true,
				},
				{
					path: 'gallery',
					name: 'Gallery',
					component: () => import('@/pages/project/Gallery.vue'),
				},
			],
		},
		{
			path: '/instance/:id',
			name: 'Instance',
			component: () => import('@/pages/instance/layout.vue'),
			children: [
				{
					path: 'worlds',
					name: 'InstanceWorlds',
					component: () => import('@/pages/instance/worlds/index.vue'),
				},
				{
					path: 'share',
					name: 'InstanceShare',
					component: () => import('@/pages/instance/share/index.vue'),
				},
				{
					path: '',
					name: 'InstanceContent',
					component: () => import('@/pages/instance/content/index.vue'),
				},
				{
					path: 'projects/:type',
					name: 'InstanceContentFilter',
					component: () => import('@/pages/instance/content/index.vue'),
				},
				{
					path: 'files',
					name: 'InstanceFiles',
					component: () => import('@/pages/instance/files/index.vue'),
				},
				{
					path: 'screenshots',
					name: 'InstanceScreenshots',
					component: () => import('@/pages/instance/screenshots/index.vue'),
				},
				{
					path: 'logs',
					name: 'InstanceLogs',
					component: () => import('@/pages/instance/logs/index.vue'),
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
		if (to.path === from.path || to.name === 'Home') return
		// Sometimes Vue's scroll behavior is not working as expected, so we need to manually scroll to top (especially on Linux)
		document.querySelector('.app-viewport')?.scrollTo(0, 0)
		return {
			el: '.app-viewport',
			top: 0,
		}
	},
})
