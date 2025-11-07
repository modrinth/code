import type { RouteRecordRaw } from 'vue-router'

export const ServersOverviewPage = () => import('./servers/manage.vue')
export { createComponentResolver, toNuxtPages } from './route-helpers'

export const sharedRoutes: RouteRecordRaw[] = [
	{
		path: '/servers/manage',
		name: 'Servers - Modrinth',
		component: ServersOverviewPage,
		meta: {
			breadcrumb: [{ name: 'Servers' }],
		},
	},
]
