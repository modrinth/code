import { defineMessages, useFormatNumber, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import type { InstallJobSnapshot, InstallPhaseId, InstallProgress } from '@/helpers/install'

const messages = defineMessages({
	paused: { id: 'app.download-manager.paused', defaultMessage: 'Paused' },
	canceling: { id: 'app.download-manager.canceling', defaultMessage: 'Canceling installation…' },
	unknownInstance: {
		id: 'app.action-bar.install.unknown-instance',
		defaultMessage: 'Unknown instance',
	},
	updatingSharedContent: {
		id: 'app.action-bar.install.updating-shared-content',
		defaultMessage: 'Updating shared content',
	},
})

const kindMessages = defineMessages({
	create_instance: { id: 'app.download-manager.new-instance', defaultMessage: 'New instance' },
	create_modpack_instance: { id: 'app.download-manager.modpack', defaultMessage: 'Modpack' },
	create_shared_instance: {
		id: 'app.download-manager.shared-instance',
		defaultMessage: 'Shared instance',
	},
	import_instance: {
		id: 'app.download-manager.imported-instance',
		defaultMessage: 'Imported instance',
	},
	duplicate_instance: {
		id: 'app.download-manager.duplicated-instance',
		defaultMessage: 'Duplicated instance',
	},
	install_existing_instance: {
		id: 'app.download-manager.instance-installation',
		defaultMessage: 'Instance installation',
	},
	install_pack_to_existing_instance: {
		id: 'app.download-manager.modpack-installation',
		defaultMessage: 'Modpack installation',
	},
	update_shared_instance: {
		id: 'app.download-manager.shared-instance-update',
		defaultMessage: 'Shared instance update',
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

export function useInstallJobDisplay() {
	const { formatMessage, locale } = useVIntl()
	const formatNumber = useFormatNumber()
	const decimalFormat = computed(
		() => new Intl.NumberFormat(locale.value, { maximumFractionDigits: 1 }),
	)
	const percentFormat = computed(() => new Intl.NumberFormat(locale.value, { style: 'percent' }))
	const units = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte']
	const byteFormats = computed(() =>
		units.map(
			(unit) =>
				new Intl.NumberFormat(locale.value, {
					style: 'unit',
					unit,
					unitDisplay: 'short',
					maximumFractionDigits: 1,
				}),
		),
	)
	const rateFormats = computed(() =>
		units.map(
			(unit) =>
				new Intl.NumberFormat(locale.value, {
					style: 'unit',
					unit: `${unit}-per-second`,
					unitDisplay: 'short',
					minimumSignificantDigits: 3,
					maximumSignificantDigits: 3,
				}),
		),
	)
	const timeFormats = computed(() =>
		['second', 'minute', 'hour'].map(
			(unit) =>
				new Intl.NumberFormat(locale.value, {
					style: 'unit',
					unit,
					unitDisplay: 'narrow',
					maximumFractionDigits: 0,
				}),
		),
	)

	function getTitle(job: InstallJobSnapshot, instanceName?: string): string {
		if (job.display?.title) return job.display.title
		if (job.details.type === 'instance') return job.details.name
		if (job.details.type === 'modpack' && job.details.title) return job.details.title
		return instanceName ?? formatMessage(messages.unknownInstance)
	}

	function getText(job: InstallJobSnapshot): string {
		if (job.status === 'succeeded') return formatMessage(kindMessages[job.kind])
		if (job.status === 'canceled') return formatMessage(failureSummaryMessages.canceled)
		if (job.status === 'failed' || job.status === 'interrupted') {
			return getFailureSummary(job)
		}
		if (job.canceling) return formatMessage(messages.canceling)
		if (job.paused) return formatMessage(messages.paused)
		if (job.phase === 'preparing_java' && job.details.type === 'java') {
			return formatMessage(javaStepMessages[job.details.step], {
				version: job.details.major_version,
			})
		}
		if (job.kind === 'update_shared_instance' && job.phase === 'downloading_content') {
			return formatMessage(messages.updatingSharedContent)
		}
		return formatMessage(phaseMessages[job.phase])
	}

	function getFailureSummary(job: InstallJobSnapshot): string {
		const code = job.error?.code
		const phase = job.error?.phase ?? job.phase

		if (code === 'app_closed' || (job.status === 'interrupted' && code === 'interrupted')) {
			return formatMessage(failureSummaryMessages.appClosed)
		}
		if (job.rollback_error || code === 'rollback_error') {
			return formatMessage(failureSummaryMessages.cleanupIncomplete)
		}
		if (code === 'canceled') {
			return formatMessage(failureSummaryMessages.canceled)
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

	function getProgressType(job: InstallJobSnapshot): 'bytes' | 'count' | 'percentage' | undefined {
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
		const progress = getEffectiveProgress(job)
		if (!progress || progress.total <= 0) return 0
		return Math.max(0, Math.min(1, progress.current / progress.total))
	}

	function getUnitIndex(bytes: number): number {
		return Math.min(units.length - 1, Math.floor(Math.log10(Math.max(1, bytes)) / 3))
	}

	function getProgressLabel(job: InstallJobSnapshot): string {
		const progress = getEffectiveProgress(job)
		if (!progress || progress.total <= 0) return ''
		const current = Math.max(0, Math.min(progress.current, progress.total))
		if (getProgressType(job) === 'bytes') {
			const unit = getUnitIndex(progress.total)
			return `${decimalFormat.value.format(current / 1000 ** unit)} / ${byteFormats.value[unit].format(progress.total / 1000 ** unit)}`
		}
		if (getProgressType(job) === 'count') {
			return `${formatNumber(current)} / ${formatNumber(progress.total)}`
		}
		return percentFormat.value.format(getProgress(job))
	}

	function formatRate(bytesPerSecond: number | null): string {
		if (bytesPerSecond == null || bytesPerSecond <= 0) return ''
		const roundedRate = Number(bytesPerSecond.toPrecision(3))
		const unit = getUnitIndex(roundedRate)
		return rateFormats.value[unit].format(roundedRate / 1000 ** unit)
	}

	function formatEta(seconds: number | null): string {
		if (seconds == null || seconds <= 0) return ''
		const unit = seconds < 60 ? 0 : seconds < 3600 ? 1 : 2
		return timeFormats.value[unit].format(Math.ceil(seconds / [1, 60, 3600][unit]))
	}

	return {
		getTitle,
		getText,
		getProgress,
		getProgressLabel,
		getEffectiveProgress,
		formatRate,
		formatEta,
	}
}
