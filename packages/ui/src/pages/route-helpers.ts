import { resolve } from 'pathe'
import type { RouteRecordRaw } from 'vue-router'

interface NuxtPage {
	name?: string
	path: string
	file: string
	children?: NuxtPage[]
	meta?: Record<string, any>
	props?: boolean | Record<string, any>
}

export function toNuxtPages(
	routes: RouteRecordRaw[],
	componentToFilePath: (component: any) => string,
): NuxtPage[] {
	return routes.map((route) => {
		const nuxtPage: NuxtPage = {
			name: route.name as string,
			path: route.path,
			file: componentToFilePath(route.component),
			meta: route.meta,
			props: route.props,
		}

		if (route.children) {
			nuxtPage.children = toNuxtPages(route.children, componentToFilePath)
		}

		return nuxtPage
	})
}

export function createComponentResolver(baseDir: string) {
	const componentMap = new Map<any, string>()

	return {
		register(component: any, relativePath: string) {
			componentMap.set(component, resolve(baseDir, relativePath))
		},
		resolve(component: any): string {
			const path = componentMap.get(component)
			if (!path) {
				throw new Error(`Component not registered: ${component}`)
			}
			return path
		},
	}
}
