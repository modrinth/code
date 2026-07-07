import { CopyIcon, UpdatedIcon } from '@modrinth/assets'
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
	type InstallPhaseId,
	type InstallJobStatus,
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
	dismiss: {
		id: 'app.action-bar.install.dismiss',
		defaultMessage: 'Dismiss',
	},
	openInstance: {
		id: 'app.action-bar.install.open-instance',
		defaultMessage: 'Open instance',
	},
	installFailed: {
		id: 'app.action-bar.install.failed',
		defaultMessage: 'Install failed',
	},
	installFailedWhile: {
		id: 'app.action-bar.install.failed-while',
		defaultMessage: 'Failed while {phase}.',
	},
	installInterruptedWhile: {
		id: 'app.action-bar.install.interrupted-while',
		defaultMessage: 'Interrupted while {phase}.',
	},
	unknownInstance: {
		id: 'app.action-bar.install.unknown-instance',
		defaultMessage: 'Unknown instance',
	},
})

const failurePhaseMessages = defineMessages({
	preparing_instance: {
		id: 'app.install.failure-phase.preparing_instance',
		defaultMessage: 'preparing instance',
	},
	resolving_pack: {
		id: 'app.install.failure-phase.resolving_pack',
		defaultMessage: 'resolving content',
	},
	downloading_pack_file: {
		id: 'app.install.failure-phase.downloading_pack_file',
		defaultMessage: 'downloading pack file',
	},
	reading_pack_manifest: {
		id: 'app.install.failure-phase.reading_pack_manifest',
		defaultMessage: 'reading pack manifest',
	},
	downloading_content: {
		id: 'app.install.failure-phase.downloading_content',
		defaultMessage: 'downloading content',
	},
	extracting_overrides: {
		id: 'app.install.failure-phase.extracting_overrides',
		defaultMessage: 'extracting overrides',
	},
	resolving_minecraft: {
		id: 'app.install.failure-phase.resolving_minecraft',
		defaultMessage: 'resolving Minecraft',
	},
	resolving_loader: {
		id: 'app.install.failure-phase.resolving_loader',
		defaultMessage: 'resolving loader',
	},
	preparing_java: {
		id: 'app.install.failure-phase.preparing_java',
		defaultMessage: 'preparing Java',
	},
	downloading_minecraft: {
		id: 'app.install.failure-phase.downloading_minecraft',
		defaultMessage: 'downloading Minecraft',
	},
	running_loader_processors: {
		id: 'app.install.failure-phase.running_loader_processors',
		defaultMessage: 'running loader processors',
	},
	finalizing: {
		id: 'app.install.failure-phase.finalizing',
		defaultMessage: 'finalizing',
	},
	rolling_back: {
		id: 'app.install.failure-phase.rolling_back',
		defaultMessage: 'rolling back',
	},
})

const phaseMessages = defineMessages({
	preparing_instance: {
		id: 'app.install.phase.preparing_instance',
		defaultMessage: 'Preparing instance',
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

const failureJavaStepMessages = defineMessages({
	resolving: {
		id: 'app.install.failure-phase.preparing_java.resolving',
		defaultMessage: 'preparing Java {version}',
	},
	fetching_metadata: {
		id: 'app.install.failure-phase.preparing_java.fetching-metadata',
		defaultMessage: 'fetching Java {version}',
	},
	downloading: {
		id: 'app.install.failure-phase.preparing_java.downloading',
		defaultMessage: 'downloading Java {version}',
	},
	extracting: {
		id: 'app.install.failure-phase.preparing_java.extracting',
		defaultMessage: 'extracting Java {version}',
	},
	validating: {
		id: 'app.install.failure-phase.preparing_java.validating',
		defaultMessage: 'validating Java {version}',
	},
})

const visibleJobStatuses = new Set<InstallJobStatus>(['queued', 'running', 'failed', 'interrupted'])

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
	const jobOrder = new Map<string, number>()
	let refreshRequest = 0
	let metadataRequest = 0
	let nextJobOrder = 0

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

	function getFailurePhase(job: InstallJobSnapshot): InstallPhaseId | undefined {
		if (job.error?.phase) {
			return job.error.phase
		}
		if (job.phase !== 'rolling_back') {
			return job.phase
		}
		return undefined
	}

	function getFailurePhaseText(job: InstallJobSnapshot, phase: InstallPhaseId): string {
		if (phase === 'preparing_java' && job.details.type === 'java') {
			return formatMessage(failureJavaStepMessages[job.details.step], {
				version: job.details.major_version,
			})
		}
		return formatMessage(failurePhaseMessages[phase])
	}

	function getText(job: InstallJobSnapshot): string {
		if (job.status === 'failed' || job.status === 'interrupted') {
			const phase = getFailurePhase(job)
			if (phase) {
				return formatMessage(
					job.status === 'interrupted'
						? messages.installInterruptedWhile
						: messages.installFailedWhile,
					{ phase: getFailurePhaseText(job, phase) },
				)
			}
			return formatMessage(messages.installFailed)
		}
		if (job.phase === 'preparing_java' && job.details.type === 'java') {
			return formatMessage(javaStepMessages[job.details.step], {
				version: job.details.major_version,
			})
		}
		return formatMessage(phaseMessages[job.phase])
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

	async function copyJobDetails(job: InstallJobSnapshot) {
		const details = await install_job_support_details(job.job_id).catch((error) => {
			opts.handleError(error)
			return null
		})
		if (!details) {
			return
		}
		await navigator.clipboard.writeText(details).catch(opts.handleError)
	}

	function getTerminalButtons(job: InstallJobSnapshot): PopupNotificationButton[] | undefined {
		if (!isTerminalJob(job)) return undefined

		return [
			{
				label: formatMessage(messages.retry),
				icon: UpdatedIcon,
				color: 'brand',
				keepOpen: true,
				action: async () => {
					await install_job_retry(job.job_id).catch(opts.handleError)
					await refresh()
				},
			},
			{
				label: formatMessage(messages.copyDetails),
				icon: CopyIcon,
				color: 'standard',
				keepOpen: true,
				action: async () => {
					await copyJobDetails(job)
				},
			},
		]
	}

	function setJobs(nextJobs: InstallJobSnapshot[]) {
		for (const job of nextJobs) {
			if (!jobOrder.has(job.job_id)) {
				jobOrder.set(job.job_id, nextJobOrder++)
			}
		}

		jobs.value = nextJobs
			.filter((job) => visibleJobStatuses.has(job.status))
			.sort(
				(a, b) =>
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
				buttons: getTerminalButtons(job),
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
		dispose: () => unlisten(),
	}
}
