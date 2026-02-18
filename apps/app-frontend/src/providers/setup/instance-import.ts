import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { provideInstanceImport } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'

import {
	get_default_launcher_path,
	get_importable_instances,
	import_instance,
} from '@/helpers/import.js'

export function setupInstanceImportProvider(notificationManager: AbstractWebNotificationManager) {
	const { handleError } = notificationManager

	provideInstanceImport({
		async getDetectedLaunchers() {
			const launcherNames = [
				'MultiMC',
				'GDLauncher',
				'ATLauncher',
				'Curseforge',
				'PrismLauncher',
			]
			const launchers = []
			for (const name of launcherNames) {
				try {
					const path = await get_default_launcher_path(name)
					if (!path) continue
					const instances = await get_importable_instances(name, path)
					if (instances?.length > 0) {
						launchers.push({ name, path, instances })
					}
				} catch {
					// Skip launchers that fail detection
				}
			}
			return launchers
		},
		async getImportableInstances(launcherName: string, path: string) {
			return (await get_importable_instances(launcherName, path)) ?? []
		},
		async importInstances(selections) {
			for (const sel of selections) {
				for (const instanceName of sel.instanceNames) {
					await import_instance(sel.launcher, sel.path, instanceName).catch(handleError)
				}
			}
		},
		async selectDirectory() {
			const result = await open({ multiple: false, directory: true })
			return result?.toString() ?? null
		},
	})
}
