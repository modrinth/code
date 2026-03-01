import { invoke } from '@tauri-apps/api/core'

import { create } from './profile'
import type { InstanceLoader } from './types'

interface PackProfileCreator {
	name: string
	gameVersion: string
	modloader: InstanceLoader
	loaderVersion: string | null
}

interface PackLocationVersionId {
	type: 'fromVersionId'
	project_id: string
	version_id: string
	title: string
	icon_url?: string
}

interface PackLocationFile {
	type: 'fromFile'
	path: string
}

export async function create_profile_and_install(
	projectId: string,
	versionId: string,
	packTitle: string,
	iconUrl?: string,
	createInstanceCallback: (profile: string) => void = () => {},
): Promise<void> {
	const location: PackLocationVersionId = {
		type: 'fromVersionId',
		project_id: projectId,
		version_id: versionId,
		title: packTitle,
		icon_url: iconUrl,
	}
	const profile_creator = await invoke<PackProfileCreator>(
		'plugin:pack|pack_get_profile_from_pack',
		{ location },
	)
	const profile = await create(
		profile_creator.name,
		profile_creator.gameVersion,
		profile_creator.modloader,
		profile_creator.loaderVersion,
		null,
		true,
	)
	createInstanceCallback(profile)

	return await invoke('plugin:pack|pack_install', { location, profile })
}

export async function install_to_existing_profile(
	projectId: string,
	versionId: string,
	title: string,
	profilePath: string,
): Promise<void> {
	const location: PackLocationVersionId = {
		type: 'fromVersionId',
		project_id: projectId,
		version_id: versionId,
		title,
	}
	return await invoke('plugin:pack|pack_install', { location, profile: profilePath })
}

export async function create_profile_and_install_from_file(path: string): Promise<void> {
	const location: PackLocationFile = {
		type: 'fromFile',
		path,
	}
	const profile_creator = await invoke<PackProfileCreator>(
		'plugin:pack|pack_get_profile_from_pack',
		{ location },
	)
	const profile = await create(
		profile_creator.name,
		profile_creator.gameVersion,
		profile_creator.modloader,
		profile_creator.loaderVersion,
		null,
		true,
	)
	return await invoke('plugin:pack|pack_install', { location, profile })
}
