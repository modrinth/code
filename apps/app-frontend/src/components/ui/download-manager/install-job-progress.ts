import type { InstallJobSnapshot, InstallPhaseId } from '@/helpers/install'

type Stage = readonly [InstallPhaseId, number]

const instanceStages: readonly Stage[] = [
	['preparing_instance', 2],
	['resolving_minecraft', 3],
	['resolving_loader', 3],
	['preparing_java', 12],
	['downloading_minecraft', 70],
	['running_loader_processors', 8],
	['finalizing', 2],
]

const packStages: readonly Stage[] = [
	['preparing_instance', 1],
	['resolving_pack', 1],
	['downloading_pack_file', 5],
	['reading_pack_manifest', 1],
	['downloading_content', 60],
	['extracting_overrides', 7],
	['resolving_minecraft', 1],
	['resolving_loader', 1],
	['preparing_java', 5],
	['downloading_minecraft', 14],
	['running_loader_processors', 3],
	['finalizing', 1],
]

const copyStages: readonly Stage[] = [
	['preparing_instance', 35],
	['resolving_minecraft', 2],
	['resolving_loader', 2],
	['preparing_java', 8],
	['downloading_minecraft', 45],
	['running_loader_processors', 7],
	['finalizing', 1],
]

const stagesByKind: Record<InstallJobSnapshot['kind'], readonly Stage[]> = {
	create_instance: instanceStages,
	create_modpack_instance: packStages,
	create_shared_instance: packStages,
	import_instance: copyStages,
	duplicate_instance: copyStages,
	install_existing_instance: instanceStages,
	install_pack_to_existing_instance: packStages,
	update_shared_instance: packStages,
}

/** Estimates whole-job progress from stage counters, preserving progress within an attempt. */
export function createInstallJobProgressTracker() {
	const jobs = new Map<string, { status: InstallJobSnapshot['status']; progress: number }>()

	function update(job: InstallJobSnapshot) {
		const previous = jobs.get(job.job_id)
		const continuing = previous?.status === 'running' || previous?.status === 'queued'
		let progress = continuing ? previous.progress : 0

		if (job.status === 'queued') {
			progress = 0
		} else if (job.status === 'succeeded') {
			progress = 1
		} else if (job.status !== 'running' || job.phase === 'rolling_back') {
			progress = previous?.progress ?? 0
		} else if (job.phase !== 'downloading_minecraft' || job.progress) {
			const counter =
				job.phase === 'downloading_content'
					? (job.progress?.secondary ?? job.progress)
					: job.progress
			const fraction =
				counter && counter.total > 0 ? Math.max(0, Math.min(1, counter.current / counter.total)) : 0
			let completedWeight = 0
			for (const [phase, weight] of stagesByKind[job.kind]) {
				if (phase === job.phase) {
					progress = Math.max(progress, Math.min(0.99, (completedWeight + weight * fraction) / 100))
					break
				}
				completedWeight += weight
			}
		}

		jobs.set(job.job_id, { status: job.status, progress })
	}

	return {
		update,
		get: (id: string) => jobs.get(id)?.progress ?? 0,
		remove: (id: string) => jobs.delete(id),
	}
}
