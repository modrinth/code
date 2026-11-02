import type { InstanceLoader } from '@/helpers/types'

export type UninstalledSharedInstance = {
	type: 'uninstalled_shared_instance'
	id: string
	shared_instance_id: string
	role: 'owner' | 'member'
	manager_id: string | null
	name: string
	icon_path?: string
	game_version: string
	loader: InstanceLoader
}
