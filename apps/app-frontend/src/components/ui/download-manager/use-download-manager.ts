import { injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, onMounted, onScopeDispose, ref, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings'
import { toError } from '@/helpers/errors'
import {
	install_job_cancel,
	install_job_dismiss,
	install_job_list,
	install_job_pause,
	install_job_resume,
	install_job_retry,
	install_job_support_details,
	installJobInstanceId,
	type InstallJobSnapshot,
} from '@/helpers/install'
import { get_many as getInstances } from '@/helpers/instance'
import { injectAppEvents } from '@/providers/app-events'

import { createDownloadTransferTracker } from './download-transfer'
import { createInstallJobProgressTracker } from './install-job-progress'
import { useInstallJobDisplay } from './use-install-job-display'

export interface DownloadManagerJob {
	id: string
	instanceId: string | null
	status: InstallJobSnapshot['status']
	paused: boolean
	canceling: boolean
	canPause: boolean
	canCancel: boolean
	title: string
	iconUrl: string | null
	text: string
	progress: number
	overallProgress: number
	progressLabel: string
	waiting: boolean
	eta: string
	canCopyDetails: boolean
	copied: boolean
	busy: boolean
}

function getIconUrl(icon: string | null | undefined): string | null {
	if (!icon) return null
	return /^(https?:|data:|blob:|asset:|tauri:)/.test(icon) ? icon : convertFileSrc(icon)
}

export function useDownloadManager() {
	const events = injectAppEvents()
	const { handleError } = injectNotificationManager()
	const appSettings = useAppSettings()
	const display = useInstallJobDisplay()
	const jobs = ref(new Map<string, InstallJobSnapshot>())
	const initialized = ref(false)
	const instances = ref(new Map<string, { name: string; icon: string | null }>())
	const busyJobs = ref(new Set<string>())
	const copiedJobs = ref(new Set<string>())
	const dismissedJobs = new Set<string>()
	const copiedTimeouts = new Map<string, ReturnType<typeof setTimeout>>()
	const transfer = createDownloadTransferTracker()
	const overallProgress = createInstallJobProgressTracker()
	const now = ref(performance.now())
	const revisions = new Map<string, number>()
	let revision = 0
	let refreshRequest = 0
	let metadataRequest = 0
	let disposed = false
	let clock: ReturnType<typeof setInterval> | undefined

	function reportError(error: unknown) {
		if (!disposed) handleError(toError(error))
	}

	function applyJobUpdate(job: InstallJobSnapshot) {
		if (disposed || dismissedJobs.has(job.job_id)) return
		const previous = jobs.value.get(job.job_id)
		if (previous && previous.modified > job.modified) return
		revisions.set(job.job_id, ++revision)
		transfer.update(job, performance.now())
		overallProgress.update(job)
		jobs.value.set(job.job_id, job)
	}

	async function refresh() {
		const request = ++refreshRequest
		const startedAtRevision = revision
		try {
			const snapshots = await install_job_list(true)
			if (disposed || request !== refreshRequest) return
			const nextJobs = new Map<string, InstallJobSnapshot>()
			for (const job of snapshots) {
				if (dismissedJobs.has(job.job_id)) continue
				const previous = jobs.value.get(job.job_id)
				if (
					previous &&
					((revisions.get(job.job_id) ?? 0) > startedAtRevision || previous.modified > job.modified)
				) {
					nextJobs.set(job.job_id, previous)
				} else {
					transfer.update(job, performance.now())
					overallProgress.update(job)
					nextJobs.set(job.job_id, job)
				}
			}
			for (const [id, job] of jobs.value) {
				if ((revisions.get(id) ?? 0) > startedAtRevision && !dismissedJobs.has(id)) {
					nextJobs.set(id, job)
				}
				if (!nextJobs.has(id)) {
					transfer.remove(id)
					overallProgress.remove(id)
				}
			}
			jobs.value = nextJobs
		} catch (error) {
			reportError(error)
		} finally {
			if (!disposed) initialized.value = true
		}
	}

	const instanceIds = computed(() =>
		Array.from(
			new Set(
				[...jobs.value.values()].map(installJobInstanceId).filter((id): id is string => !!id),
			),
		).sort(),
	)

	async function refreshMetadata() {
		const request = ++metadataRequest
		try {
			const metadata = instanceIds.value.length ? await getInstances(instanceIds.value) : []
			if (disposed || request !== metadataRequest) return
			instances.value = new Map(
				metadata.map((instance) => [
					instance.id,
					{ name: instance.name, icon: getIconUrl(instance.icon_path) },
				]),
			)
		} catch (error) {
			reportError(error)
		}
	}

	watch(() => instanceIds.value.join('\n'), refreshMetadata)

	const rows = computed(() =>
		[...jobs.value.values()].map((job): DownloadManagerJob => {
			const instanceId = installJobInstanceId(job)
			const instance = instanceId ? instances.value.get(instanceId) : undefined
			const progress = display.getEffectiveProgress(job)
			return {
				id: job.job_id,
				instanceId: instance && instanceId ? instanceId : null,
				status: job.status,
				paused: job.paused,
				canceling: job.canceling,
				canPause: job.can_pause,
				canCancel: job.can_cancel,
				title: display.getTitle(job, instance?.name),
				iconUrl: getIconUrl(job.display?.icon) ?? instance?.icon ?? null,
				text: display.getText(job),
				progress: display.getProgress(job),
				overallProgress: overallProgress.get(job.job_id),
				progressLabel: display.getProgressLabel(job),
				waiting: !progress || progress.total <= 0,
				eta:
					job.paused || job.canceling
						? ''
						: display.formatEta(transfer.get(job.job_id, now.value).eta),
				canCopyDetails:
					job.status === 'failed' ||
					job.status === 'interrupted' ||
					appSettings.getFeatureFlag('always_show_copy_details'),
				copied: copiedJobs.value.has(job.job_id),
				busy: busyJobs.value.has(job.job_id),
			}
		}),
	)

	function newestFirst(a: DownloadManagerJob, b: DownloadManagerJob) {
		const first = jobs.value.get(a.id)!
		const second = jobs.value.get(b.id)!
		return (second.finished ?? second.modified).localeCompare(first.finished ?? first.modified)
	}

	const activeJobs = computed(() =>
		rows.value
			.filter((job) => job.status === 'queued' || job.status === 'running')
			.sort(
				(a, b) =>
					Number(a.status === 'queued') - Number(b.status === 'queued') ||
					jobs.value.get(a.id)!.created.localeCompare(jobs.value.get(b.id)!.created),
			),
	)
	const attentionJobs = computed(() =>
		rows.value
			.filter((job) => job.status === 'failed' || job.status === 'interrupted')
			.sort(newestFirst),
	)
	const completedJobs = computed(() =>
		rows.value
			.filter((job) => job.status === 'succeeded' || job.status === 'canceled')
			.sort(newestFirst),
	)
	const rate = computed(() =>
		display.formatRate(
			activeJobs.value.reduce(
				(total, job) => total + (transfer.get(job.id, now.value).rate ?? 0),
				0,
			),
		),
	)

	watch(
		() => activeJobs.value.length > 0,
		(active) => {
			if (clock) clearInterval(clock)
			clock = active
				? setInterval(() => {
						now.value = performance.now()
					}, 1000)
				: undefined
		},
	)

	async function runAction(id: string, action: () => Promise<unknown>) {
		if (disposed || busyJobs.value.has(id)) return
		busyJobs.value.add(id)
		try {
			await action()
		} catch (error) {
			reportError(error)
		} finally {
			busyJobs.value.delete(id)
		}
	}

	async function retry(id: string) {
		await runAction(id, async () => {
			const before = revision
			const job = await install_job_retry(id)
			if ((revisions.get(id) ?? 0) <= before) applyJobUpdate(job)
		})
	}

	async function cancel(id: string) {
		if (!jobs.value.get(id)?.can_cancel) return
		await runAction(id, async () => {
			const before = revision
			const job = await install_job_cancel(id)
			if ((revisions.get(id) ?? 0) <= before) applyJobUpdate(job)
		})
	}

	async function togglePause(id: string) {
		const current = jobs.value.get(id)
		if (!current?.can_pause) return
		await runAction(id, async () => {
			const before = revision
			const job = await (current.paused ? install_job_resume(id) : install_job_pause(id))
			if ((revisions.get(id) ?? 0) <= before) applyJobUpdate(job)
		})
	}

	async function dismiss(id: string) {
		const job = jobs.value.get(id)
		if (!job || job.status === 'queued' || job.status === 'running') return
		await runAction(id, async () => {
			await install_job_dismiss(id)
			dismissedJobs.add(id)
			jobs.value.delete(id)
			transfer.remove(id)
			overallProgress.remove(id)
		})
	}

	async function copyDetails(id: string) {
		await runAction(id, async () => {
			const details = await install_job_support_details(id)
			if (disposed) return
			await navigator.clipboard.writeText(details)
			if (disposed) return
			copiedJobs.value.add(id)
			clearTimeout(copiedTimeouts.get(id))
			copiedTimeouts.set(
				id,
				setTimeout(() => {
					copiedJobs.value.delete(id)
					copiedTimeouts.delete(id)
				}, 1500),
			)
		})
	}

	const unlisten = events.on('install_job', applyJobUpdate)
	const unlistenInstance = events.on('instance', () => {
		void refreshMetadata()
	})
	onMounted(() => {
		void refresh()
	})
	onScopeDispose(() => {
		disposed = true
		unlisten()
		unlistenInstance()
		if (clock) clearInterval(clock)
		for (const timeout of copiedTimeouts.values()) clearTimeout(timeout)
	})

	return {
		activeJobs,
		attentionJobs,
		completedJobs,
		initialized,
		rate,
		retry,
		cancel,
		togglePause,
		dismiss,
		copyDetails,
	}
}
