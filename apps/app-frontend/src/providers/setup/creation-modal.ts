import type { AbstractWebNotificationManager, CreationFlowContextValue } from '@modrinth/ui'
import { provide, useTemplateRef } from 'vue'
import { useRouter } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_search_results } from '@/helpers/cache.js'
import { import_instance } from '@/helpers/import.js'
import { create_profile_and_install, create_profile_and_install_from_file } from '@/helpers/pack'
import { create, list } from '@/helpers/profile.js'

export function setupCreationModal(notificationManager: AbstractWebNotificationManager) {
	const { handleError } = notificationManager
	const router = useRouter()

	const installationModal = useTemplateRef('installationModal')
	provide('showCreationModal', async () => {
		const instances = await list().catch(handleError)
		installationModal.value?.show(instances?.length ?? 0)
	})

	async function handleCreate(config: CreationFlowContextValue) {
		installationModal.value?.hide()

		try {
			if (config.isImportMode.value) {
				for (const [launcherName, instanceSet] of Object.entries(
					config.importSelectedInstances.value,
				)) {
					const launcher = config.importLaunchers.value.find((l) => l.name === launcherName)
					if (!launcher || instanceSet.size === 0) continue
					for (const name of instanceSet) {
						await import_instance(launcher.name, launcher.path, name).catch(handleError)
					}
				}
				trackEvent('InstanceCreate', { source: 'CreationModalImport' })
				return
			}

			if (config.modpackSelection.value) {
				const { projectId, versionId, name, iconUrl } = config.modpackSelection.value
				await create_profile_and_install(projectId, versionId, name, iconUrl).catch(handleError)
				trackEvent('InstanceCreate', { source: 'CreationModalModpack' })
				return
			}

			if (config.modpackFilePath.value) {
				await create_profile_and_install_from_file(config.modpackFilePath.value).catch(handleError)
				trackEvent('InstanceCreate', { source: 'CreationModalModpackFile' })
				return
			}

			// Custom/vanilla setup
			const loader = config.hideLoaderChips.value
				? 'vanilla'
				: (config.selectedLoader.value ?? 'vanilla')
			const loaderVersion = config.hideLoaderVersion.value
				? null
				: (config.selectedLoaderVersion.value ?? config.loaderVersionType.value)
			const iconPath = config.instanceIconPath.value ?? null

			await create(
				config.instanceName.value,
				config.selectedGameVersion.value,
				loader,
				loaderVersion,
				iconPath,
				false,
			).catch(handleError)

			trackEvent('InstanceCreate', {
				profile_name: config.instanceName.value,
				game_version: config.selectedGameVersion.value,
				loader,
				loader_version: loaderVersion,
				has_icon: !!iconPath,
				source: 'CreationModal',
			})
		} catch (err) {
			handleError(err)
		}
	}

	function handleBrowseModpacks() {
		installationModal.value?.hide()
		router.push('/browse/modpack')
	}

	async function searchModpacks(query: string, limit: number = 10) {
		const params = [`facets=[["project_type:modpack"]]`, `limit=${limit}`]
		if (query) {
			params.push(`query=${encodeURIComponent(query)}`)
		}
		const raw = await get_search_results(`?${params.join('&')}`)
		if (raw?.result) return raw.result
		return { hits: [], offset: 0, limit, total_hits: 0 }
	}

	async function getProjectVersions(projectId: string) {
		const versions = await get_project_versions(projectId)
		return versions ?? []
	}

	return {
		installationModal,
		handleCreate,
		handleBrowseModpacks,
		searchModpacks,
		getProjectVersions,
	}
}
