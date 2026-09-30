import {
	commonMessages,
	defineMessages,
	injectNotificationManager,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
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
import {
	externalFileDownloads,
	type ExternalFileDownloadTask,
	getExternalFileDownloadRate,
} from './external-file-downloads'
import { createInstallJobProgressTracker } from './install-job-progress'
import { storeVerificationTask } from './store-verification'
import { useInstallJobDisplay } from './use-install-job-display'

export interface DownloadManagerJob {
	id: string
	kind?: 'external-file'
	serverId?: string
	createdAt?: string
	instanceId: string | null
	status: InstallJobSnapshot['status']
	paused: boolean
	canceling: boolean
	canPause: boolean
	canCancel: boolean
	title: string
	iconUrl: string | null
	text: string
	taskType?: string
	finishedAt?: string
	progress: number
	overallProgress: number
	progressLabel: string
	waiting: boolean
	eta: string
	canRetry: boolean
	canCopyDetails: boolean
	copied: boolean
	busy: boolean
}

const verificationMessages = defineMessages({
	verifying: { id: 'app.download-manager.verifying', defaultMessage: 'Verifying' },
	title: { id: 'app.settings.resource-management.store.title', defaultMessage: 'Content storage' },
	complete: {
		id: 'app.settings.resource-management.store.verified',
		defaultMessage: 'Verification complete',
	},
	failed: {
		id: 'app.settings.resource-management.store.attention',
		defaultMessage: 'Some files still need attention',
	},
})

const fileMessages = defineMessages({
	preparing: { id: 'app.file-download.preparing', defaultMessage: 'Preparing download…' },
	waiting: { id: 'app.file-download.waiting', defaultMessage: 'Waiting for the server…' },
	canceling: { id: 'app.file-download.canceling', defaultMessage: 'Canceling download…' },
	canceled: { id: 'app.action-bar.install.summary.canceled', defaultMessage: 'Canceled' },
})

function getIconUrl(icon: string | null | undefined): string | null {
	if (!icon) return null
	return /^(https?:|data:|blob:|asset:|tauri:)/.test(icon) ? icon : convertFileSrc(icon)
}

export function useDownloadManager() {
	const events = injectAppEvents()
	const { handleError } = injectNotificationManager()
	const appSettings = useAppSettings()
	const display = useInstallJobDisplay()
	const { formatMessage } = useVIntl()
	const formatBytes = useFormatBytes()
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

	const instanceIds = computed(() =>
		Array.from(
			new Set(
				[...jobs.value.values()].map(installJobInstanceId).filter((id): id is string => !!id),
			),
		).sort(),
	)

	const rows = computed(() =>
		[...jobs.value.values()].map((job): DownloadManagerJob => {
			const instanceId = installJobInstanceId(job)
			const instance = instanceId ? instances.value.get(instanceId) : undefined
			const progress = display.getEffectiveProgress(job)
			return {
				id: job.job_id,
				createdAt: job.created,
				instanceId: instance && instanceId ? instanceId : null,
				status: job.status,
				paused: job.paused,
				canceling: job.canceling,
				canPause: job.can_pause,
				canCancel: job.can_cancel,
				title: display.getTitle(job, instance?.name),
				iconUrl: getIconUrl(job.display?.icon) ?? instance?.icon ?? null,
				text: display.getText(job),
				taskType: job.kind === 'bulk_update_content' ? display.getTaskType(job) : undefined,
				finishedAt: job.finished ?? job.modified,
				progress: display.getProgress(job),
				overallProgress: overallProgress.get(job.job_id),
				progressLabel: [
					display.getProgressLabel(job),
					job.kind === 'bulk_update_content'
						? display.formatRate(transfer.get(job.job_id, now.value).rate)
						: '',
				]
					.filter(Boolean)
					.join(' · '),
				waiting: !progress || progress.total <= 0,
				eta:
					job.paused || job.canceling
						? ''
						: display.formatEta(transfer.get(job.job_id, now.value).eta),
				canRetry: job.status === 'failed' || job.status === 'interrupted',
				canCopyDetails:
					job.status === 'failed' ||
					job.status === 'interrupted' ||
					appSettings.alwaysShowCopyDetails,
				copied: copiedJobs.value.has(job.job_id),
				busy: busyJobs.value.has(job.job_id),
			}
		}),
	)

	const verificationRows = computed(() => {
		const task = storeVerificationTask.value
		return task ? [buildVerificationRow(task)] : []
	})
	const fileRows = computed(() => [...externalFileDownloads.value.values()].map(buildFileRow))
	const allRows = computed(() => [...rows.value, ...verificationRows.value, ...fileRows.value])

	const activeJobs = computed(() =>
		allRows.value
			.filter((job) => job.status === 'queued' || job.status === 'running')
			.sort(
				(a, b) =>
					Number(a.status === 'queued') - Number(b.status === 'queued') ||
					(a.createdAt ?? '').localeCompare(b.createdAt ?? ''),
			),
	)
	const attentionJobs = computed(() =>
		allRows.value
			.filter((job) => job.status === 'failed' || job.status === 'interrupted')
			.sort(newestFirst),
	)
	const completedJobs = computed(() =>
		allRows.value
			.filter((job) => job.status === 'succeeded' || job.status === 'canceled')
			.sort(newestFirst),
	)
	const rate = computed(() =>
		display.formatRate(
			activeJobs.value.reduce(
				(total, job) => {
					const file = externalFileDownloads.value.get(job.id)
					const fileRate = file ? getExternalFileDownloadRate(file, now.value) : null
					return total + (fileRate ?? transfer.get(job.id, now.value).rate ?? 0)
				},
				0,
			),
		),
	)

	function buildFileRow(task: ExternalFileDownloadTask): DownloadManagerJob {
		const rate = getExternalFileDownloadRate(task, now.value)
		const progress = task.totalBytes ? Math.min(0.99, task.downloadedBytes / task.totalBytes) : 0
		const text =
			task.error ??
			formatMessage(
				task.canceling
					? fileMessages.canceling
					: task.status === 'canceled'
						? fileMessages.canceled
						: task.status === 'succeeded'
							? commonMessages.savedLabel
							: task.stage === 'preparing'
								? fileMessages.preparing
								: task.stage === 'waiting'
									? fileMessages.waiting
									: task.stage === 'saving'
										? commonMessages.savingButton
										: commonMessages.downloadingButton,
			)

		return {
			id: task.id,
			kind: 'external-file',
			serverId: task.serverId,
			createdAt: task.createdAt,
			instanceId: null,
			status: task.status,
			paused: false,
			canceling: task.canceling,
			canPause: false,
			canCancel: task.status === 'running' && task.stage !== 'saving',
			title: task.serverName ?? task.filename,
			iconUrl: null,
			text,
			taskType: task.filename,
			finishedAt: task.finishedAt,
			progress,
			overallProgress: task.status === 'succeeded' ? 1 : progress,
			progressLabel:
				task.status === 'running' && (task.stage === 'downloading' || task.stage === 'saving')
					? [
							task.totalBytes
								? `${formatBytes(task.downloadedBytes)} / ${formatBytes(task.totalBytes)}`
								: formatBytes(task.downloadedBytes),
							display.formatRate(rate),
						]
							.filter(Boolean)
							.join(' · ')
					: '',
			waiting: task.stage !== 'downloading' || !task.totalBytes,
			eta:
				rate && task.totalBytes
					? display.formatEta((task.totalBytes - task.downloadedBytes) / rate)
					: '',
			canRetry: false,
			canCopyDetails: !!task.error,
			copied: copiedJobs.value.has(task.id),
			busy: busyJobs.value.has(task.id),
		}
	}

	function buildVerificationRow(
		task: NonNullable<typeof storeVerificationTask.value>,
	): DownloadManagerJob {
		const progress = task.total ? Math.min(0.99, task.current / task.total) : 0
		const rate = task.status === 'running' && now.value - task.lastRead < 2000 ? task.rate : 0

		return {
			id: task.id,
			instanceId: null,
			status: task.status,
			paused: false,
			canceling: false,
			canPause: false,
			canCancel: false,
			canRetry: false,
			title: formatMessage(verificationMessages.title),
			iconUrl: null,
			text: formatMessage(
				task.status === 'running'
					? verificationMessages.verifying
					: task.status === 'succeeded'
						? verificationMessages.complete
						: verificationMessages.failed,
			),
			progress,
			overallProgress: task.status === 'succeeded' ? 1 : progress,
			progressLabel:
				task.status === 'running'
					? `${formatBytes(task.current)} / ${formatBytes(task.total)} · ${display.formatRate(rate) || '0 B/s'}`
					: '',
			waiting: task.total === 0 || task.current >= task.total,
			eta: '',
			canCopyDetails: false,
			copied: false,
			busy: false,
		}
	}

	function newestFirst(a: DownloadManagerJob, b: DownloadManagerJob) {
		return (b.finishedAt ?? '').localeCompare(a.finishedAt ?? '')
	}

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

	/** Job events received while an action is pending take precedence over its response. */
	async function runJobAction(id: string, action: () => Promise<InstallJobSnapshot>) {
		await runAction(id, async () => {
			const before = revision
			const job = await action()
			if ((revisions.get(id) ?? 0) <= before) applyJobUpdate(job)
		})
	}

	async function retry(id: string) {
		await runJobAction(id, () => install_job_retry(id))
	}

	async function cancel(id: string) {
		const file = externalFileDownloads.value.get(id)
		if (file) {
			await runAction(id, file.cancel)
			return
		}
		if (!jobs.value.get(id)?.can_cancel) return
		await runJobAction(id, () => install_job_cancel(id))
	}

	async function togglePause(id: string) {
		const current = jobs.value.get(id)
		if (!current?.can_pause) return
		await runJobAction(id, () => (current.paused ? install_job_resume(id) : install_job_pause(id)))
	}

	async function dismiss(id: string) {
		const file = externalFileDownloads.value.get(id)
		if (file) {
			if (file.status !== 'running') externalFileDownloads.value.delete(id)
			return
		}
		if (storeVerificationTask.value?.id === id) {
			if (storeVerificationTask.value.status !== 'running') storeVerificationTask.value = null
			return
		}
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

	async function clearCompleted() {
		await Promise.all(completedJobs.value.map((job) => dismiss(job.id)))
	}

	async function copyDetails(id: string) {
		await runAction(id, async () => {
			const file = externalFileDownloads.value.get(id)
			const details = file ? file.error ?? '' : await install_job_support_details(id)
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

	watch(() => instanceIds.value.join('\n'), refreshMetadata)
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
		{ immediate: true },
	)

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
		clearCompleted,
		copyDetails,
	}
}
