import { onScopeDispose, type Ref, ref, shallowRef, watch } from 'vue'

import type { DownloadManagerJob } from './use-download-manager'

export function useDownloadBarState(options: {
	activeJobs: Ref<DownloadManagerJob[]>
	attentionJobs: Ref<DownloadManagerJob[]>
	completedJobs: Ref<DownloadManagerJob[]>
	initialized: Ref<boolean>
}) {
	const task = shallowRef<DownloadManagerJob | null>(null)
	const completing = ref(false)
	let hydrated = false
	let completionTimer: ReturnType<typeof setTimeout> | undefined
	let queuedAtCompletion = new Set<string>()

	function clearCompletion() {
		clearTimeout(completionTimer)
		completionTimer = undefined
		completing.value = false
	}

	function selectTask() {
		task.value = options.activeJobs.value[0] ?? options.attentionJobs.value[0] ?? null
	}

	function reconcile() {
		if (!options.initialized.value || !hydrated) {
			hydrated = options.initialized.value
			selectTask()
			return
		}

		const current = task.value
		const active = options.activeJobs.value.find((job) => job.id === current?.id)
		if (active) {
			clearCompletion()
			task.value = active
			return
		}

		if (completing.value) {
			const hasNewTask = options.activeJobs.value.some((job) => !queuedAtCompletion.has(job.id))
			const completed = options.completedJobs.value.find((job) => job.id === current?.id)
			if (hasNewTask || !completed || completed.status !== 'succeeded') {
				clearCompletion()
				selectTask()
			}
			return
		}

		if (current?.status === 'running' || current?.status === 'queued') {
			const completed = options.completedJobs.value.find(
				(job) => job.id === current.id && job.status === 'succeeded',
			)
			if (completed) {
				task.value = completed
				completing.value = true
				queuedAtCompletion = new Set(options.activeJobs.value.map((job) => job.id))
				completionTimer = setTimeout(
					() => {
						clearCompletion()
						selectTask()
					},
					options.activeJobs.value.length ? 500 : 1000,
				)
				return
			}
		}

		selectTask()
	}

	watch(
		[options.activeJobs, options.attentionJobs, options.completedJobs, options.initialized],
		reconcile,
		{ immediate: true },
	)
	onScopeDispose(clearCompletion)

	return { task, completing }
}
