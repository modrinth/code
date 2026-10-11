import type { Archon, UploadProgress } from '@modrinth/api-client'

import { createContext } from '.'

export interface ServerInstanceOption {
	id: string
	name: string
	iconUrl?: string
}

/**
 * Lets hosting screens create and update a server from one of the user's instances. Only the
 * App provides this, so inject it with a `null` fallback and hide the feature when it is absent.
 */
export interface InstanceServerSourceProvider {
	listInstances: () => Promise<ServerInstanceOption[]>
	/**
	 * Exports the instance and uploads it to the server. The first push installs it and links
	 * the server to the instance; later pushes update it. With `restart`, a running server is
	 * restarted to apply the update now instead of on its next stop.
	 */
	pushInstance: (args: {
		serverId: string
		worldId: string
		instanceId: string
		properties?: Archon.Content.v1.PropertiesFields
		restart?: boolean
		onProgress?: (progress: UploadProgress) => void
	}) => Promise<void>
}

export const [injectInstanceServerSource, provideInstanceServerSource] =
	createContext<InstanceServerSourceProvider>('root', 'instanceServerSource')
