import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { EditingFile } from '../types'

export function useFileRouteState() {
	const route = useRoute()
	const router = useRouter()

	const currentPath = computed(() =>
		typeof route.query.path === 'string' ? route.query.path : '/',
	)

	const editingFile = computed<EditingFile | null>(() => {
		const path = route.query.editing
		if (typeof path !== 'string' || !path) return null
		return { name: path.split('/').pop() || '', path }
	})

	function navigateTo(path: string) {
		const { editing: _, ...query } = route.query
		router.push({ query: { ...query, path } })
	}

	function startEditing(file: EditingFile) {
		router.push({ query: { ...route.query, path: currentPath.value, editing: file.path } })
	}

	function stopEditing() {
		const { editing: _, ...query } = route.query
		router.push({ query })
	}

	return { currentPath, editingFile, navigateTo, startEditing, stopEditing }
}
