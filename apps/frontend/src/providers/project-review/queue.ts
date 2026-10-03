import { injectModrinthClient } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'
import type { ModerationQueueService } from '~/services/moderation/queue'
import { findNextEligibleQueueProject } from '~/services/moderation/queue-eligibility'

export function useReviewQueue(
	projectId: Ref<string>,
	queue: ModerationQueueService,
	beforeNavigate: () => Promise<boolean>,
) {
	const router = useRouter()
	const route = useRoute()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const busy = ref(false)
	const error = ref<unknown>(null)
	const canGoBack = computed(() => queue.currentQueue.history.length > 0)
	const inQueue = computed(
		() =>
			queue.isQueueMode &&
			[
				...queue.currentQueue.items,
				...queue.currentQueue.completed,
				...queue.currentQueue.skipped,
			].includes(projectId.value),
	)
	const completed = computed(() => queue.currentQueue.completed.includes(projectId.value))
	const remaining = computed(() =>
		inQueue.value ? queue.currentQueue.items.filter((id) => id !== projectId.value) : [],
	)

	async function run(action: () => Promise<unknown>) {
		if (busy.value) return
		busy.value = true
		error.value = null
		try {
			if (!(await beforeNavigate())) return
			await action()
		} catch (cause) {
			error.value = cause
		} finally {
			busy.value = false
		}
	}

	async function open(id: string, back = false) {
		const failure = await router.replace({
			path: '/moderation/project-review',
			query: { ...route.query, project: id },
		})
		if (failure) throw failure
		await queue.visitProject(id, back)
	}

	async function advance(completedProjectId?: string) {
		if (completedProjectId && completedProjectId !== projectId.value) return
		await run(async () => {
			const id = projectId.value
			if (!id || !inQueue.value || (completedProjectId && completedProjectId !== id)) return
			if (completedProjectId && !completed.value) await queue.completeProject(id)
			const candidates = remaining.value
			const next = await findNextEligibleQueueProject(client, queue, candidates, true)
			if (next) await queryClient.fetchQuery(projectQueryOptions.v3(next.project, client))
			if (!completed.value) await queue.deferProject(id)
			for (const excluded of next?.excluded ?? candidates) await queue.excludeProject(excluded)
			if (next) {
				await open(next.project)
			} else {
				await router.push({
					path: '/moderation',
					state: { queueSummary: true },
				})
			}
		})
	}

	async function next() {
		await advance()
	}

	async function completeAndNext(id: string) {
		await advance(id)
	}

	async function back() {
		await run(async () => {
			const id = queue.currentQueue.history.at(-1)
			if (!id) return
			await queryClient.fetchQuery(projectQueryOptions.v3(id, client))
			await open(id, true)
		})
	}

	async function exit() {
		await run(async () => {
			await router.push('/moderation')
		})
	}

	return {
		busy,
		error,
		canGoBack,
		inQueue,
		completed,
		remaining,
		next,
		completeAndNext,
		back,
		exit,
	}
}
