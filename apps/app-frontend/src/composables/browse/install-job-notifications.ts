import { CheckIcon, CopyIcon, UpdatedIcon } from '@modrinth/assets'
import {
	defineMessages,
	type PopupNotificationButton,
	type PopupNotificationProgressItem,
	type PopupNotificationProgressType,
	useVIntl,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'
import type { Router } from 'vue-router'

import { install_job_listener } from '@/helpers/events'
import {
	install_job_dismiss,
	install_job_list,
	install_job_retry,
	install_job_support_details,
	installJobInstanceId,
	type InstallJobSnapshot,
	type InstallJobStatus,
	type InstallPhaseId,
	type InstallProgress,
} from '@/helpers/install'
import { get_many as getInstances } from '@/helpers/instance'

const messages = defineMessages({
	installs: {
		id: 'app.action-bar.installs',
		defaultMessage: 'Installs',
	},
	retry: {
		id: 'app.action-bar.install.retry',
		defaultMessage: 'Retry',
	},
	copyDetails: {
		id: 'app.action-bar.install.copy-details',
		defaultMessage: 'Copy details',
	},
	copied: {
		id: 'app.action-bar.install.copied-details',
		defaultMessage: 'Copied',
	},
	dismiss: {
		id: 'app.action-bar.install.dismiss',
		defaultMessage: 'Dismiss',
	},
	openInstance: {
		id: 'app.action-bar.install.open-instance',
		defaultMessage: 'Open instance',
	},
	unknownInstance: {
		id: 'app.action-bar.install.unknown-instance',
		defaultMessage: 'Unknown instance',
	},
})

const phaseMessages = defineMessages({
	preparing_instance: {
		id: 'app.install.phase.preparing_instance',
		defaultMessage: 'Queued to install',
	},
	resolving_pack: {
		id: 'app.install.phase.resolving_pack',
		defaultMessage: 'Resolving content',
	},
	downloading_pack_file: {
		id: 'app.install.phase.downloading_pack_file',
		defaultMessage: 'Downloading pack file',
	},
	reading_pack_manifest: {
		id: 'app.install.phase.reading_pack_manifest',
		defaultMessage: 'Reading pack manifest',
	},
	downloading_content: {
		id: 'app.install.phase.downloading_content',
		defaultMessage: 'Downloading content',
	},
	extracting_overrides: {
		id: 'app.install.phase.extracting_overrides',
		defaultMessage: 'Extracting overrides',
	},
	resolving_minecraft: {
		id: 'app.install.phase.resolving_minecraft',
		defaultMessage: 'Resolving Minecraft',
	},
	resolving_loader: {
		id: 'app.install.phase.resolving_loader',
		defaultMessage: 'Resolving loader',
	},
	preparing_java: {
		id: 'app.install.phase.preparing_java',
		defaultMessage: 'Preparing Java',
	},
	downloading_minecraft: {
		id: 'app.install.phase.downloading_minecraft',
		defaultMessage: 'Downloading Minecraft',
	},
	running_loader_processors: {
		id: 'app.install.phase.running_loader_processors',
		defaultMessage: 'Running loader processors',
	},
	finalizing: {
		id: 'app.install.phase.finalizing',
		defaultMessage: 'Finalizing',
	},
	rolling_back: {
		id: 'app.install.phase.rolling_back',
		defaultMessage: 'Rolling back',
	},
})

const javaStepMessages = defineMessages({
	resolving: {
		id: 'app.install.phase.preparing_java.resolving',
		defaultMessage: 'Preparing Java {version}',
	},
	fetching_metadata: {
		id: 'app.install.phase.preparing_java.fetching-metadata',
		defaultMessage: 'Fetching Java {version}',
	},
	downloading: {
		id: 'app.install.phase.preparing_java.downloading',
		defaultMessage: 'Downloading Java {version}',
	},
	extracting: {
		id: 'app.install.phase.preparing_java.extracting',
		defaultMessage: 'Extracting Java {version}',
	},
	validating: {
		id: 'app.install.phase.preparing_java.validating',
		defaultMessage: 'Validating Java {version}',
	},
})

const failureSummaryMessages = defineMessages({
	canceled: {
		id: 'app.action-bar.install.summary.canceled',
		defaultMessage: 'Canceled',
	},
	appClosed: {
		id: 'app.action-bar.install.summary.app-closing',
		defaultMessage: 'Canceled due to app closing',
	},
	downloadFailed: {
		id: 'app.action-bar.install.summary.download-failed',
		defaultMessage: "Download couldn't finish",
	},
	modrinthUnreachable: {
		id: 'app.action-bar.install.summary.modrinth-unreachable',
		defaultMessage: "Couldn't reach Modrinth",
	},
	packDownloadFailed: {
		id: 'app.action-bar.install.summary.pack-download-failed',
		defaultMessage: "Couldn't download pack",
	},
	badModpackFile: {
		id: 'app.action-bar.install.summary.bad-modpack-file',
		defaultMessage: "Couldn't read modpack",
	},
	invalidModpack: {
		id: 'app.action-bar.install.summary.invalid-modpack',
		defaultMessage: 'Modpack data invalid',
	},
	contentDownloadFailed: {
		id: 'app.action-bar.install.summary.content-download-failed',
		defaultMessage: "Couldn't download files",
	},
	corruptDownload: {
		id: 'app.action-bar.install.summary.corrupt-download',
		defaultMessage: 'Downloaded file is corrupt',
	},
	invalidModpackFiles: {
		id: 'app.action-bar.install.summary.invalid-modpack-files',
		defaultMessage: 'Modpack files have invalid metadata',
	},
	noWritePermission: {
		id: 'app.action-bar.install.summary.no-write-permission',
		defaultMessage: 'No permission to write',
	},
	couldNotSaveFiles: {
		id: 'app.action-bar.install.summary.could-not-save-files',
		defaultMessage: "Couldn't save files",
	},
	invalidFilePath: {
		id: 'app.action-bar.install.summary.invalid-file-path',
		defaultMessage: 'File path is invalid',
	},
	instanceNotFound: {
		id: 'app.action-bar.install.summary.instance-not-found',
		defaultMessage: "Instance couldn't be found",
	},
	cleanupIncomplete: {
		id: 'app.action-bar.install.summary.cleanup-incomplete',
		defaultMessage: "Cleanup didn't finish",
	},
	javaSetupFailed: {
		id: 'app.action-bar.install.summary.java-setup-failed',
		defaultMessage: "Java setup couldn't finish",
	},
	minecraftSetupFailed: {
		id: 'app.action-bar.install.summary.minecraft-setup-failed',
		defaultMessage: 'Minecraft setup failed',
	},
	loaderSetupFailed: {
		id: 'app.action-bar.install.summary.loader-setup-failed',
		defaultMessage: 'Loader setup failed',
	},
	localDataError: {
		id: 'app.action-bar.install.summary.local-data-error',
		defaultMessage: "Couldn't update local data",
	},
	unexpectedError: {
		id: 'app.action-bar.install.summary.unexpected-error',
		defaultMessage: 'Something went wrong',
	},
})

const visibleJobStatuses = new Set<InstallJobStatus>(['queued', 'running', 'failed', 'interrupted'])
const copyDetailsStallMs = 30_000

function getDisplayIconUrl(icon: string | null | undefined): string | null {
	if (!icon) return null
	if (/^(https?:|data:|blob:|asset:|tauri:)/.test(icon)) return icon
	return convertFileSrc(icon)
}

export async function useInstallJobNotifications(opts: {
	router: Router
	handleError: (err: unknown) => void
	onChange: () => void
}) {
	const { formatMessage } = useVIntl()
	const jobs = ref<InstallJobSnapshot[]>([])
	const iconUrls = ref<Record<string, string | null>>({})
	const instanceNames = ref<Record<string, string>>({})
	const copiedJobIds = ref<Set<string>>(new Set())
	const jobOrder = new Map<string, number>()
	let refreshRequest = 0
	let metadataRequest = 0
	let nextJobOrder = 0
	const progressSnapshots = new Map<string, { signature: string; changedAt: number }>()
	const copiedResetTimeouts = new Map<string, number>()
	const staleProgressTick = ref(Date.now())
	const staleProgressInterval = window.setInterval(() => {
		if (jobs.value.some((job) => isActiveProgressJob(job))) {
			staleProgressTick.value = Date.now()
			opts.onChange()
		}
	}, 1_000)

	function getTitle(job: InstallJobSnapshot): string {
		if (job.display?.title) return job.display.title
		if (job.details.type === 'instance') return job.details.name
		if (job.details.type === 'modpack' && job.details.title) return job.details.title
		const instanceId = installJobInstanceId(job)
		return (
			(instanceId ? instanceNames.value[instanceId] : null) ??
			formatMessage(messages.unknownInstance)
		)
	}

	function getText(job: InstallJobSnapshot): string {
		if (job.status === 'failed' || job.status === 'interrupted') {
			return getFailureSummary(job)
		}
		if (job.phase === 'preparing_java' && job.details.type === 'java') {
			return formatMessage(javaStepMessages[job.details.step], {
				version: job.details.major_version,
			})
		}
		return formatMessage(phaseMessages[job.phase])
	}

	function getFailureSummary(job: InstallJobSnapshot): string {
		const code = job.error?.code
		const phase = job.error?.phase ?? job.phase

		if (code === 'app_closed' || (job.status === 'interrupted' && code === 'interrupted')) {
			return formatMessage(failureSummaryMessages.appClosed)
		}
		if (code === 'canceled') {
			return formatMessage(failureSummaryMessages.canceled)
		}
		if (job.rollback_error || code === 'rollback_error') {
			return formatMessage(failureSummaryMessages.cleanupIncomplete)
		}
		if (hasPermissionError(job)) {
			return formatMessage(failureSummaryMessages.noWritePermission)
		}

		switch (code) {
			case 'network_error':
				return formatMessage(
					phase === 'downloading_pack_file'
						? failureSummaryMessages.packDownloadFailed
						: failureSummaryMessages.downloadFailed,
				)
			case 'api_error':
				return formatMessage(failureSummaryMessages.modrinthUnreachable)
			case 'pack_error':
				return formatMessage(
					phase === 'downloading_pack_file'
						? failureSummaryMessages.packDownloadFailed
						: failureSummaryMessages.invalidModpack,
				)
			case 'archive_error':
				return formatMessage(failureSummaryMessages.badModpackFile)
			case 'parse_error':
				return formatMessage(failureSummaryMessages.invalidModpack)
			case 'content_error':
				return formatMessage(failureSummaryMessages.invalidModpackFiles)
			case 'hash_error':
				return formatMessage(failureSummaryMessages.corruptDownload)
			case 'filesystem_error':
				return formatMessage(failureSummaryMessages.couldNotSaveFiles)
			case 'path_error':
				return formatMessage(failureSummaryMessages.invalidFilePath)
			case 'instance_error':
				return formatMessage(failureSummaryMessages.instanceNotFound)
			case 'java_error':
				return formatMessage(failureSummaryMessages.javaSetupFailed)
			case 'loader_error':
			case 'processor_error':
				return formatMessage(failureSummaryMessages.loaderSetupFailed)
			case 'database_error':
				return formatMessage(failureSummaryMessages.localDataError)
			case 'launcher_error':
			case 'metadata_error':
				return getFailureSummaryForPhase(phase)
			default:
				return getFailureSummaryForPhase(phase)
		}
	}

	function getFailureSummaryForPhase(phase: InstallPhaseId): string {
		switch (phase) {
			case 'downloading_pack_file':
				return formatMessage(failureSummaryMessages.packDownloadFailed)
			case 'resolving_pack':
			case 'reading_pack_manifest':
				return formatMessage(failureSummaryMessages.invalidModpack)
			case 'downloading_content':
				return formatMessage(failureSummaryMessages.contentDownloadFailed)
			case 'extracting_overrides':
				return formatMessage(failureSummaryMessages.couldNotSaveFiles)
			case 'resolving_minecraft':
			case 'downloading_minecraft':
				return formatMessage(failureSummaryMessages.minecraftSetupFailed)
			case 'resolving_loader':
			case 'running_loader_processors':
				return formatMessage(failureSummaryMessages.loaderSetupFailed)
			case 'preparing_java':
				return formatMessage(failureSummaryMessages.javaSetupFailed)
			case 'preparing_instance':
				return formatMessage(failureSummaryMessages.instanceNotFound)
			case 'rolling_back':
				return formatMessage(failureSummaryMessages.cleanupIncomplete)
			default:
				return formatMessage(failureSummaryMessages.unexpectedError)
		}
	}

	function hasPermissionError(job: InstallJobSnapshot): boolean {
		const message = job.error?.message.toLowerCase() ?? ''
		return (
			message.includes('permission denied') ||
			message.includes('access is denied') ||
			message.includes('operation not permitted')
		)
	}

	function getProgressType(job: InstallJobSnapshot): PopupNotificationProgressType | undefined {
		if (!getEffectiveProgress(job)) return undefined
		if (
			job.phase === 'preparing_java' &&
			job.details.type === 'java' &&
			job.details.step === 'downloading'
		) {
			return 'bytes'
		}
		if (job.phase === 'downloading_content') {
			return job.progress?.secondary ? 'bytes' : 'count'
		}
		if (
			job.phase === 'downloading_pack_file' ||
			job.phase === 'extracting_overrides' ||
			job.phase === 'downloading_minecraft'
		) {
			return 'bytes'
		}
		if (job.phase === 'running_loader_processors') {
			return 'count'
		}
		return 'percentage'
	}

	function getEffectiveProgress(job: InstallJobSnapshot): InstallProgress | null | undefined {
		if (job.phase === 'downloading_content' && job.progress?.secondary) {
			return job.progress.secondary
		}

		return job.progress
	}

	function getProgress(job: InstallJobSnapshot): number {
		if (job.status === 'succeeded') return 1
		if (job.status === 'failed' || job.status === 'interrupted') return 0
		const progress = getEffectiveProgress(job)
		if (!progress || progress.total <= 0) return 0
		return Math.max(0, Math.min(1, progress.current / progress.total))
	}

	function isTerminalJob(job: InstallJobSnapshot): boolean {
		return job.status === 'failed' || job.status === 'interrupted'
	}

	function isActiveProgressJob(job: InstallJobSnapshot): boolean {
		return (
			(job.status === 'queued' || job.status === 'running') &&
			job.phase !== 'finalizing' &&
			job.phase !== 'rolling_back'
		)
	}

	function getJobSortRank(job: InstallJobSnapshot): number {
		if (isTerminalJob(job)) return 0
		if (job.status === 'queued' || job.phase === 'preparing_instance') return 2
		return 1
	}

	function progressSignature(job: InstallJobSnapshot): string {
		const progress = job.progress
		const secondary = progress?.secondary
		return [
			job.status,
			job.phase,
			JSON.stringify(job.details),
			progress?.current ?? '',
			progress?.total ?? '',
			secondary?.current ?? '',
			secondary?.total ?? '',
		].join(':')
	}

	function syncProgressSnapshots(nextJobs: InstallJobSnapshot[]) {
		const activeIds = new Set(nextJobs.map((job) => job.job_id))
		const now = Date.now()

		for (const job of nextJobs) {
			const signature = progressSignature(job)
			const snapshot = progressSnapshots.get(job.job_id)
			if (!snapshot || snapshot.signature !== signature) {
				progressSnapshots.set(job.job_id, { signature, changedAt: now })
				if (copiedJobIds.value.has(job.job_id)) {
					const nextCopiedJobIds = new Set(copiedJobIds.value)
					nextCopiedJobIds.delete(job.job_id)
					copiedJobIds.value = nextCopiedJobIds
				}
			}
		}

		for (const jobId of progressSnapshots.keys()) {
			if (!activeIds.has(jobId)) {
				progressSnapshots.delete(jobId)
			}
		}
	}

	function hasStalledProgress(job: InstallJobSnapshot): boolean {
		const snapshot = progressSnapshots.get(job.job_id)
		return !!snapshot && staleProgressTick.value - snapshot.changedAt >= copyDetailsStallMs
	}

	function shouldShowCopyDetails(job: InstallJobSnapshot): boolean {
		if (job.status === 'queued' || job.phase === 'preparing_instance') return false
		return isTerminalJob(job) || (isActiveProgressJob(job) && hasStalledProgress(job))
	}

	function isCopied(job: InstallJobSnapshot): boolean {
		return copiedJobIds.value.has(job.job_id)
	}

	function setCopied(job: InstallJobSnapshot) {
		copiedJobIds.value = new Set([...copiedJobIds.value, job.job_id])
		const existingTimeout = copiedResetTimeouts.get(job.job_id)
		if (existingTimeout != null) {
			window.clearTimeout(existingTimeout)
		}
		copiedResetTimeouts.set(
			job.job_id,
			window.setTimeout(() => {
				copiedResetTimeouts.delete(job.job_id)
				if (!copiedJobIds.value.has(job.job_id)) {
					return
				}
				const nextCopiedJobIds = new Set(copiedJobIds.value)
				nextCopiedJobIds.delete(job.job_id)
				copiedJobIds.value = nextCopiedJobIds
				opts.onChange()
			}, 1_000),
		)
		opts.onChange()
	}

	async function copyJobDetails(job: InstallJobSnapshot) {
		const details = await install_job_support_details(job.job_id).catch((error) => {
			opts.handleError(error)
			return null
		})
		if (!details) {
			return
		}
		await navigator.clipboard
			.writeText(details)
			.then(() => setCopied(job))
			.catch(opts.handleError)
	}

	function getButtons(job: InstallJobSnapshot): PopupNotificationButton[] {
		const buttons: PopupNotificationButton[] = []

		if (isTerminalJob(job)) {
			buttons.push({
				label: formatMessage(messages.retry),
				icon: UpdatedIcon,
				color: 'brand',
				keepOpen: true,
				action: async () => {
					await install_job_retry(job.job_id).catch(opts.handleError)
					await refresh()
				},
			})
		}

		if (shouldShowCopyDetails(job)) {
			const copied = isCopied(job)
			buttons.push({
				label: formatMessage(copied ? messages.copied : messages.copyDetails),
				icon: copied ? CheckIcon : CopyIcon,
				color: 'standard',
				keepOpen: true,
				action: async () => {
					await copyJobDetails(job)
				},
			})
		}

		return buttons
	}

	function setJobs(nextJobs: InstallJobSnapshot[]) {
		for (const job of nextJobs) {
			if (!jobOrder.has(job.job_id)) {
				jobOrder.set(job.job_id, nextJobOrder++)
			}
		}

		const visibleJobs = nextJobs.filter((job) => visibleJobStatuses.has(job.status))
		syncProgressSnapshots(visibleJobs)

		jobs.value = visibleJobs.sort(
			(a, b) =>
				getJobSortRank(a) - getJobSortRank(b) ||
				a.created.localeCompare(b.created) ||
				(jobOrder.get(a.job_id) ?? 0) - (jobOrder.get(b.job_id) ?? 0),
		)
	}

	const progressItems = computed<PopupNotificationProgressItem[]>(() =>
		jobs.value.map((job) => {
			const progress = getEffectiveProgress(job)

			return {
				id: job.job_id,
				title: getTitle(job),
				text: getText(job),
				iconUrl: iconUrls.value[job.job_id] ?? null,
				progress: getProgress(job),
				waiting: !job.progress && ['queued', 'running'].includes(job.status),
				showProgress: !isTerminalJob(job),
				wrapText: isTerminalJob(job),
				progressType: isTerminalJob(job) ? undefined : getProgressType(job),
				progressCurrent: isTerminalJob(job) ? undefined : progress?.current,
				progressTotal: isTerminalJob(job) ? undefined : progress?.total,
				buttons: getButtons(job),
				onDismiss: isTerminalJob(job)
					? async () => {
							await install_job_dismiss(job.job_id).catch(opts.handleError)
							await refresh()
						}
					: undefined,
			}
		}),
	)

	const buttons = computed<PopupNotificationButton[] | undefined>(() => undefined)

	async function refreshMetadata(notify = true) {
		const request = ++metadataRequest
		const sourceJobs = jobs.value
		const instanceIds = Array.from(
			new Set(
				sourceJobs
					.map((job) => installJobInstanceId(job))
					.filter((instanceId): instanceId is string => !!instanceId),
			),
		)
		const instances = instanceIds.length
			? await getInstances(instanceIds).catch((error) => {
					opts.handleError(error)
					return []
				})
			: []

		if (request !== metadataRequest) {
			return
		}

		const instanceIconUrls = new Map(
			instances.map((instance) => [instance.id, getDisplayIconUrl(instance.icon_path)]),
		)
		instanceNames.value = Object.fromEntries(
			instances.map((instance) => [instance.id, instance.name]),
		)
		iconUrls.value = Object.fromEntries(
			sourceJobs.map((job) => [
				job.job_id,
				getDisplayIconUrl(job.display?.icon) ??
					instanceIconUrls.get(installJobInstanceId(job) ?? '') ??
					null,
			]),
		)

		if (notify) {
			opts.onChange()
		}
	}

	async function refresh(notify = true) {
		const request = ++refreshRequest
		const nextJobs = await install_job_list(false).catch((error) => {
			opts.handleError(error)
			return []
		})

		if (request !== refreshRequest) {
			return
		}

		setJobs(nextJobs)
		await refreshMetadata(false)

		if (request !== refreshRequest) {
			return
		}

		if (notify) {
			opts.onChange()
		}
	}

	function applyJobUpdate(job: InstallJobSnapshot) {
		refreshRequest += 1
		const existingJob = jobs.value.find((item) => item.job_id === job.job_id)
		if (existingJob && existingJob.modified.localeCompare(job.modified) > 0) {
			return
		}

		setJobs([...jobs.value.filter((item) => item.job_id !== job.job_id), job])
		opts.onChange()
		void refreshMetadata()
	}

	await refresh(false)
	const unlisten = await install_job_listener((job: InstallJobSnapshot) => applyJobUpdate(job))

	return {
		active: computed(() => jobs.value.length > 0),
		title: computed(() => formatMessage(messages.installs)),
		progressItems,
		buttons,
		refresh,
		dispose: () => {
			window.clearInterval(staleProgressInterval)
			for (const timeout of copiedResetTimeouts.values()) {
				window.clearTimeout(timeout)
			}
			unlisten()
		},
	}
}
