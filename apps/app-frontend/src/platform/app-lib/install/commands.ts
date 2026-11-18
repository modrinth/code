import { invoke } from '@tauri-apps/api/core'

export interface InstallJobSupportDetails {
	details: string
	finished_modified: number | null
}

export function getInstallJobSupportDetails(jobId: string): Promise<InstallJobSupportDetails> {
	return invoke('plugin:install|install_job_support_details', { jobId })
}

export function getInstallJobSupportPasteUrl(jobId: string): Promise<string | null> {
	return invoke('plugin:install|install_job_support_paste_url', { jobId })
}

export function setInstallJobSupportPasteUrl(
	jobId: string,
	url: string,
	finishedModified: number,
): Promise<void> {
	return invoke('plugin:install|install_job_set_support_paste_url', {
		jobId,
		url,
		finishedModified,
	})
}
