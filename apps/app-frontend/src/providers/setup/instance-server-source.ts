import { provideInstanceServerSource } from '@modrinth/ui'

import { getInstanceIconUrl, list } from '@/helpers/instance'
import { exportInstancePack, pushPackToServer } from '@/helpers/linked-servers'

/** Lets the hosting screens create and update a server from one of this App's instances. */
export function setupInstanceServerSourceProvider() {
	provideInstanceServerSource({
		async listInstances() {
			return (await list())
				.filter((instance) => instance.install_stage === 'installed' && !instance.quarantined)
				.map((instance) => ({
					id: instance.id,
					name: instance.name,
					iconUrl: getInstanceIconUrl(instance.icon_path) ?? undefined,
				}))
		},
		async pushInstance({ serverId, instanceId, properties, restart, onProgress }) {
			const instance = (await list()).find((candidate) => candidate.id === instanceId)
			if (!instance) {
				throw new Error('This instance is not on this device, so it cannot be pushed from here.')
			}
			const pack = await exportInstancePack(instance.id)
			await pushPackToServer(pack, { id: instance.id, name: instance.name }, serverId, {
				properties,
				restart,
				onProgress,
			})
		},
	})
}
