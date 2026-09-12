import type { InstallJobSnapshot } from '@/helpers/install'

interface TransferSample {
	phase: string
	current: number
	total: number
	at: number
	rate: number | null
}

function getDownloadProgress(job: InstallJobSnapshot) {
	if (job.status !== 'running' || job.paused || job.canceling) return null
	if (job.phase === 'downloading_content') return job.progress?.secondary ?? null
	if (
		job.phase === 'downloading_pack_file' ||
		job.phase === 'downloading_minecraft' ||
		(job.phase === 'preparing_java' &&
			job.details.type === 'java' &&
			job.details.step === 'downloading')
	) {
		return job.progress
	}
	return null
}

export function createDownloadTransferTracker() {
	const samples = new Map<string, TransferSample>()

	function update(job: InstallJobSnapshot, now: number) {
		const progress = getDownloadProgress(job)
		if (!progress || progress.total <= 0 || progress.current >= progress.total) {
			samples.delete(job.job_id)
			return
		}

		const previous = samples.get(job.job_id)
		if (
			!previous ||
			previous.phase !== job.phase ||
			previous.total !== progress.total ||
			progress.current < previous.current
		) {
			samples.set(job.job_id, { ...progress, phase: job.phase, at: now, rate: null })
			return
		}

		const elapsed = now - previous.at
		if (progress.current === previous.current || elapsed < 250) return
		const rate = ((progress.current - previous.current) * 1000) / elapsed
		samples.set(job.job_id, {
			...progress,
			phase: job.phase,
			at: now,
			rate: previous.rate == null || elapsed >= 5000 ? rate : previous.rate * 0.6 + rate * 0.4,
		})
	}

	function get(jobId: string, now: number) {
		const sample = samples.get(jobId)
		const rate = sample && now - sample.at < 5000 ? sample.rate : null
		return {
			rate,
			eta: rate && sample ? (sample.total - sample.current) / rate : null,
		}
	}

	return { update, get, remove: (jobId: string) => samples.delete(jobId) }
}
