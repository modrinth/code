import { UpdatedIcon } from '@modrinth/assets'
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
	installJobInstanceId,
	type InstallJobSnapshot,
	type InstallJobStatus,
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
	installFailedAppClosed: {
		id: 'app.action-bar.install.failed-app-closed',
		defaultMessage: 'Installation failed due to app closing.',
	},
	installFailedNetwork: {
		id: 'app.action-bar.install.failed-network',
		defaultMessage: 'Installation failed due to a network error.',
	},
	installFailedUnknown: {
		id: 'app.action-bar.install.failed-unknown',
		defaultMessage: 'Installation failed due to an unknown error.',
	},
	unknownInstance: {
		id: 'app.action-bar.install.unknown-instance',
		defaultMessage: 'Unknown instance',
	},
})

const phaseMessages = defineMessages({
	preparing_instance: {
		id: 'app.install.phase.preparing_instance',
		defaultMessage: 'Preparing instance',
	},
	resolving_pack: {
		id: 'app.install.phase.resolving_pack',
		defaultMessage: 'Resolving pack',
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

	function getText(job: InstallJobSnapshot): string {
		if (job.status === 'failed' || job.status === 'interrupted') {
			if (job.error?.code === 'interrupted') {
				return formatMessage(messages.installFailedAppClosed)
			}
			if (job.error?.code === 'network_error') {
				return formatMessage(messages.installFailedNetwork)
			}
			return formatMessage(messages.installFailedUnknown)
		}
		return formatMessage(phaseMessages[job.phase])
	}

	function getProgressType(job: InstallJobSnapshot): PopupNotificationProgressType | undefined {
		if (!job.progress) return undefined
		if (job.phase === 'downloading_pack_file' || job.phase === 'extracting_overrides') {
			return 'bytes'
		}
		if (job.phase === 'downloading_content' || job.phase === 'running_loader_processors') {
			return 'count'
		}
		return 'percentage'
	}

	function getProgress(job: InstallJobSnapshot): number {
		if (job.status === 'succeeded') return 1
		if (job.status === 'failed' || job.status === 'interrupted') return 0
		const progress = job.progress
		if (!progress || progress.total <= 0) return 0
		return Math.max(0, Math.min(1, progress.current / progress.total))
	}

	function isTerminalJob(job: InstallJobSnapshot): boolean {
		return job.status === 'failed' || job.status === 'interrupted'
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
		jobs.value.map((job) => ({
			id: job.job_id,
			title: getTitle(job),
			text: getText(job),
			iconUrl: iconUrls.value[job.job_id] ?? null,
			progress: getProgress(job),
			waiting: !job.progress && ['queued', 'running'].includes(job.status),
			showProgress: !isTerminalJob(job),
			wrapText: isTerminalJob(job),
			progressType: isTerminalJob(job) ? undefined : getProgressType(job),
			progressCurrent: isTerminalJob(job) ? undefined : job.progress?.current,
			progressTotal: isTerminalJob(job) ? undefined : job.progress?.total,
			buttons: getTerminalButtons(job),
			onDismiss: isTerminalJob(job)
				? async () => {
						await install_job_dismiss(job.job_id).catch(opts.handleError)
						await refresh()
					}
				: undefined,
		})),
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
