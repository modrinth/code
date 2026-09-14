import { onScopeDispose, type Ref, ref, shallowRef, watch } from 'vue'

import type { DownloadManagerJob } from './use-download-manager'

export function useDownloadBarState(options: {
	activeJobs: Ref<DownloadManagerJob[]>
	attentionJobs: Ref<DownloadManagerJob[]>
	completedJobs: Ref<DownloadManagerJob[]>
	initialized: Ref<boolean>
}) {
	const selectedJob = shallowRef<DownloadManagerJob | null>(null)
	const completing = ref(false)
	let hydrated = false
	let completionTimer: ReturnType<typeof setTimeout> | undefined
	let activeJobIdsAtCompletion = new Set<string>()

	function clearCompletion() {
		clearTimeout(completionTimer)
		completionTimer = undefined
		completing.value = false
	}

	function selectJob() {
		selectedJob.value = options.activeJobs.value[0] ?? options.attentionJobs.value[0] ?? null
	}

	function showCompletion(job: DownloadManagerJob) {
		selectedJob.value = job
		completing.value = true
		activeJobIdsAtCompletion = new Set(options.activeJobs.value.map((job) => job.id))
		completionTimer = setTimeout(
			() => {
				clearCompletion()
				selectJob()
			},
			options.activeJobs.value.length ? 500 : 1000,
		)
	}

	function reconcile() {
		if (!options.initialized.value || !hydrated) {
			hydrated = options.initialized.value
			selectJob()
			return
		}

		const current = selectedJob.value
		const active = options.activeJobs.value.find((job) => job.id === current?.id)
		if (active) {
			clearCompletion()
			selectedJob.value = active
			return
		}

		if (completing.value) {
			const hasNewJob = options.activeJobs.value.some(
				(job) => !activeJobIdsAtCompletion.has(job.id),
			)
			const completed = options.completedJobs.value.find((job) => job.id === current?.id)
			if (hasNewJob || !completed || completed.status !== 'succeeded') {
				clearCompletion()
				selectJob()
			}
			return
		}

		if (current?.status === 'running' || current?.status === 'queued') {
			const completed = options.completedJobs.value.find(
				(job) => job.id === current.id && job.status === 'succeeded',
			)
			if (completed) {
				showCompletion(completed)
				return
			}
		}

		selectJob()
	}

	watch(
		[options.activeJobs, options.attentionJobs, options.completedJobs, options.initialized],
		reconcile,
		{ immediate: true },
	)
	onScopeDispose(clearCompletion)

	return { selectedJob, completing }
}
