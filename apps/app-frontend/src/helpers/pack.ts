import { invoke } from '@tauri-apps/api/core'

import { create } from './instance'
import type { InstanceLoader } from './types'

interface PackInstanceCreator {
	name: string
	gameVersion: string
	modloader: InstanceLoader
	loaderVersion: string | null
	unknownFile: boolean
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

export async function create_instance_and_install(
	projectId: string,
	versionId: string,
	packTitle: string,
	iconUrl?: string,
	createInstanceCallback: (instanceId: string) => void = () => {},
): Promise<void> {
	const location: PackLocationVersionId = {
		type: 'fromVersionId',
		project_id: projectId,
		version_id: versionId,
		title: packTitle,
		icon_url: iconUrl,
	}
	const instanceCreator = await invoke<PackInstanceCreator>(
		'plugin:pack|pack_get_instance_from_pack',
		{ location },
	)
	const instanceId = await create(
		instanceCreator.name,
		instanceCreator.gameVersion,
		instanceCreator.modloader,
		instanceCreator.loaderVersion,
		null,
		true,
		{
			type: 'modrinth_modpack',
			project_id: projectId,
			version_id: versionId,
		},
	)
	createInstanceCallback(instanceId)

	return await invoke('plugin:pack|pack_install', { location, instanceId })
}

export async function install_to_existing_instance(
	projectId: string,
	versionId: string,
	title: string,
	instanceId: string,
): Promise<void> {
	const location: PackLocationVersionId = {
		type: 'fromVersionId',
		project_id: projectId,
		version_id: versionId,
		title,
	}
	return await invoke('plugin:pack|pack_install', { location, instanceId })
}

export async function create_instance_and_install_from_file(
	path: string,
	showUnknownPackWarningModal?: (createInstance: () => Promise<void>, fileName: string) => void,
): Promise<void> {
	const location: PackLocationFile = {
		type: 'fromFile',
		path,
	}
	const instanceCreator = await invoke<PackInstanceCreator>(
		'plugin:pack|pack_get_instance_from_pack',
		{ location },
	)

	const createInstance = async () => {
		const instanceId = await create(
			instanceCreator.name,
			instanceCreator.gameVersion,
			instanceCreator.modloader,
			instanceCreator.loaderVersion,
			null,
			true,
		)
		await invoke('plugin:pack|pack_install', { location, instanceId })
	}

	if (instanceCreator.unknownFile && showUnknownPackWarningModal) {
		const splitPath = path.split(/[\\/]/)
		const fileName = splitPath ? splitPath[splitPath.length - 1] : path
		showUnknownPackWarningModal(createInstance, fileName)
	} else {
		await createInstance()
	}
}
