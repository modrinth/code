import { createContext } from '@modrinth/ui'
import { computed, onMounted, watch } from 'vue'

import { useModerationQueue } from '~/services/moderation/queue'

import { useReviewProject } from './project'
import { useReviewQueue } from './queue'
import type { ProjectReviewPageContext } from './types'

export const [injectProjectReviewPageContext, provideProjectReviewPageContext] =
	createContext<ProjectReviewPageContext>('ProjectReviewPage')

export function createProjectReviewPageContext() {
	const route = useRoute()
	const router = useRouter()
	const queue = useModerationQueue()
	const selection = computed(() =>
		typeof route.query.project === 'string' ? route.query.project : '',
	)
	const data = useReviewProject(selection)
	const navigation = useReviewQueue(data.projectId, queue)

	onMounted(async () => {
		await queue.ready
		if (selection.value) return
		const id = queue.currentQueue.activeProjectId ?? queue.getCurrentProjectId()
		if (id) await router.replace({ query: { ...route.query, project: id } })
	})
	watch(data.projectId, async (id) => {
		if (!id || navigation.busy.value) return
		await queue.visitProject(id)
	})

	return {
		...data,
		selection,
		queue,
		navigation,
	}
}
