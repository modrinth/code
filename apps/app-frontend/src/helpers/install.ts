import { invoke } from '@tauri-apps/api/core'

import { install_job_listener } from './events'
import type { InstanceLink, InstanceLoader } from './types'

export interface PackLocationVersionId {
	type: 'fromVersionId'
	project_id: string
	version_id: string
	title: string
	icon_url?: string | null
}

export interface PackLocationFile {
	type: 'fromFile'
	path: string
}

export type CreatePackLocation = PackLocationVersionId | PackLocationFile

export interface InstallModpackPreview {
	name: string
	gameVersion: string
	modloader: InstanceLoader
	loaderVersion: string | null
	icon?: string | null
	iconUrl?: string | null
	link?: InstanceLink | null
	unknownFile: boolean
	externalFilesInModpack: string[]
}

export interface InstallCreateInstanceRequest {
	name: string
	gameVersion: string
	loader: InstanceLoader
	loaderVersion: string | null
	iconPath: string | null
	link?: InstanceLink | null
}

export interface InstallPostInstallEdit {
	name?: string | null
	iconPath?: string | null
	link?: InstanceLink | null
}

export interface SharedInstanceInstallPreview {
	name: string
	iconUrl?: string | null
	gameVersion: string
	loader: InstanceLoader
	modCount: number
	externalFileCount: number
	modpackVersionId?: string | null
	contentVersionIds: string[]
	externalFiles: SharedInstanceExternalFilePreview[]
}

export interface SharedInstanceInviteInstallPreview {
	sharedInstanceId: string
	managerId?: string | null
	serverManagerName?: string | null
	serverManagerIconUrl?: string | null
	instanceIconUrl?: string | null
	preview: SharedInstanceInstallPreview
}

export interface SharedInstanceExternalFilePreview {
	fileName: string
	fileType: string
}

export interface SharedInstanceUpdatePreview {
	sharedInstanceId: string
	currentVersion?: number | null
	latestVersion: number
	updateAvailable: boolean
	diffs: SharedInstanceUpdateDiff[]
}

export interface SharedInstanceUpdateDiff {
	type:
		| 'added'
		| 'removed'
		| 'updated'
		| 'modpack_linked'
		| 'modpack_updated'
		| 'modpack_unlinked'
		| 'game_version_updated'
		| 'loader_updated'
		| 'config_files_updated'
	projectId?: string | null
	projectName?: string | null
	fileName?: string | null
	currentVersionName?: string | null
	newVersionName?: string | null
	configFileCount?: number | null
	disabled?: boolean
}

export const SHARED_INSTANCE_UNAVAILABLE_ERROR_CODE = 'shared_instance_unavailable'

export type SharedInstanceUnavailableReason = 'deleted' | 'access_revoked'

function isRecord(value: unknown): value is Record<string, unknown> {
	return typeof value === 'object' && value !== null
}

export function isSharedInstanceUnavailableError(error: unknown) {
	return getSharedInstanceUnavailableReason(error) !== null
}

export function getSharedInstanceUnavailableReason(
	error: unknown,
): SharedInstanceUnavailableReason | null {
	if (!isRecord(error) || error.code !== SHARED_INSTANCE_UNAVAILABLE_ERROR_CODE) return null
	return error.reason === 'deleted' || error.reason === 'access_revoked' ? error.reason : null
}

export function getErrorMessage(error: unknown): string {
	if (typeof error === 'string') return error
	if (error instanceof Error) return error.message || 'Unknown error'
	if (isRecord(error) && typeof error.message === 'string') return error.message
	return 'Unknown error'
}

export type InstallJobStatus =
	| 'queued'
	| 'running'
	| 'succeeded'
	| 'failed'
	| 'interrupted'
	| 'canceled'

export type InstallPhaseId =
	| 'preparing_instance'
	| 'resolving_pack'
	| 'downloading_pack_file'
	| 'reading_pack_manifest'
	| 'downloading_content'
	| 'extracting_overrides'
	| 'resolving_minecraft'
	| 'resolving_loader'
	| 'preparing_java'
	| 'downloading_minecraft'
	| 'running_loader_processors'
	| 'finalizing'
	| 'rolling_back'

export interface InstallProgress {
	current: number
	total: number
	secondary?: InstallProgressSecondary | null
}

export interface InstallProgressSecondary {
	current: number
	total: number
}

export type InstallJavaStep =
	| 'resolving'
	| 'fetching_metadata'
	| 'downloading'
	| 'extracting'
	| 'validating'

export interface InstallErrorView {
	code: string
	phase?: InstallPhaseId | null
	message: string
	reason?: SharedInstanceUnavailableReason | null
	api?: {
		error: string
		status?: number | null
		method?: string | null
		url?: string | null
		route?: string | null
	} | null
	context?: {
		operation: string
		source_path?: string | null
		target_path?: string | null
		file_path?: string | null
		entry_path?: string | null
		urls?: string[]
		expected_hash?: string | null
		expected_size?: number | null
		project_id?: string | null
		version_id?: string | null
		minecraft_version?: string | null
		loader?: string | null
		java_version?: number | null
		os?: string | null
		arch?: string | null
	} | null
}

export interface InstallJobSnapshot {
	job_id: string
	instance_id?: string | null
	kind:
		| 'create_instance'
		| 'create_modpack_instance'
		| 'create_shared_instance'
		| 'update_shared_instance'
		| 'import_instance'
		| 'duplicate_instance'
		| 'install_existing_instance'
		| 'install_pack_to_existing_instance'
	status: InstallJobStatus
	target:
		| { type: 'new_instance'; instance_id?: string | null }
		| { type: 'existing_instance'; instance_id: string }
	phase: InstallPhaseId
	progress?: InstallProgress | null
	details:
		| { type: 'empty' }
		| { type: 'instance'; name: string }
		| { type: 'minecraft'; game_version: string; loader: InstanceLoader }
		| { type: 'java'; major_version: number; step: InstallJavaStep }
		| {
				type: 'modpack'
				project_id?: string | null
				version_id?: string | null
				title?: string | null
		  }
		| { type: 'import'; launcher_type: string; instance_folder: string }
	display?: { title: string; icon?: string | null } | null
	error?: InstallErrorView | null
	rollback_error?: InstallErrorView | null
	created: string
	modified: string
	finished?: string | null
}

export async function install_get_modpack_preview(location: CreatePackLocation) {
	return await invoke<InstallModpackPreview>('plugin:install|install_get_modpack_preview', {
		location,
	})
}

export async function install_create_instance(request: InstallCreateInstanceRequest) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_create_instance', { request })
}

export async function install_create_modpack_instance(
	location: CreatePackLocation,
	postInstallEdit?: InstallPostInstallEdit | null,
) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_create_modpack_instance', {
		location,
		postInstallEdit,
	})
}

export async function install_get_shared_instance_preview(sharedInstanceId: string, name: string) {
	return await invoke<SharedInstanceInstallPreview>(
		'plugin:install|install_get_shared_instance_preview',
		{
			sharedInstanceId,
			name,
		},
	)
}

export async function install_accept_shared_instance_invite(inviteId: string) {
	return await invoke<SharedInstanceInviteInstallPreview>(
		'plugin:install|install_accept_shared_instance_invite',
		{
			inviteId,
		},
	)
}

export async function install_get_shared_instance_update_preview(instanceId: string) {
	return await invoke<SharedInstanceUpdatePreview | null>(
		'plugin:install|install_get_shared_instance_update_preview',
		{
			instanceId,
		},
	)
}

export async function install_shared_instance(
	sharedInstanceId: string,
	name: string,
	managerId?: string | null,
	serverManagerName?: string | null,
	serverManagerIconUrl?: string | null,
	instanceIconUrl?: string | null,
) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_shared_instance', {
		sharedInstanceId,
		name,
		managerId,
		serverManagerName,
		serverManagerIconUrl,
		instanceIconUrl,
	})
}

export async function install_update_shared_instance(instanceId: string) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_update_shared_instance', {
		instanceId,
	})
}

export async function install_import_instance(
	launcherType: string,
	basePath: string,
	instanceFolder: string,
) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_import_instance', {
		launcherType,
		basePath,
		instanceFolder,
	})
}

export async function install_duplicate_instance(sourceInstanceId: string) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_duplicate_instance', {
		sourceInstanceId,
	})
}

export async function install_existing_instance(instanceId: string, force: boolean) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_existing_instance', {
		instanceId,
		force,
	})
}

export async function install_pack_to_existing_instance(
	instanceId: string,
	location: CreatePackLocation,
	postInstallEdit?: InstallPostInstallEdit | null,
) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_pack_to_existing_instance', {
		instanceId,
		location,
		postInstallEdit,
	})
}

export async function install_job_list(includeFinished: boolean) {
	return await invoke<InstallJobSnapshot[]>('plugin:install|install_job_list', { includeFinished })
}

export async function install_job_get(jobId: string) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_job_get', { jobId })
}

export async function install_job_retry(jobId: string) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_job_retry', { jobId })
}

export async function install_job_cancel(jobId: string) {
	return await invoke<InstallJobSnapshot>('plugin:install|install_job_cancel', { jobId })
}

export async function install_job_dismiss(jobId: string) {
	return await invoke<void>('plugin:install|install_job_dismiss', { jobId })
}

export async function install_job_support_details(jobId: string) {
	return await invoke<string>('plugin:install|install_job_support_details', { jobId })
}

export function installJobInstanceId(job: InstallJobSnapshot): string | null {
	return job.instance_id ?? job.target.instance_id ?? null
}

export function isInstallJobFinished(status: InstallJobStatus) {
	return (
		status === 'succeeded' ||
		status === 'failed' ||
		status === 'interrupted' ||
		status === 'canceled'
	)
}

function settleInstallJob(job: InstallJobSnapshot) {
	if (job.status === 'succeeded') return job

	if (job.error) throw job.error
	throw new Error(`Install job ${job.job_id} ${job.status}`)
}

export async function wait_for_install_job(jobId: string) {
	const current = await install_job_get(jobId)
	if (isInstallJobFinished(current.status)) return settleInstallJob(current)

	return await new Promise<InstallJobSnapshot>((resolve, reject) => {
		let finished = false
		let unlisten: (() => void) | null = null

		const cleanup = () => {
			if (unlisten) {
				unlisten()
				unlisten = null
			}
		}

		const resolveJob = (job: InstallJobSnapshot) => {
			if (finished || job.job_id !== jobId || !isInstallJobFinished(job.status)) return

			finished = true
			cleanup()

			try {
				resolve(settleInstallJob(job))
			} catch (err) {
				reject(err)
			}
		}

		const rejectWait = (err: unknown) => {
			if (finished) return
			finished = true
			cleanup()
			reject(err)
		}

		install_job_listener(resolveJob)
			.then((listener) => {
				if (finished) {
					listener()
					return
				}

				unlisten = listener
				install_job_get(jobId).then(resolveJob).catch(rejectWait)
			})
			.catch(rejectWait)
	})
}
