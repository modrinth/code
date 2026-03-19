import type {
	AbstractWebNotificationManager,
	CreationFlowContextValue,
	CreationFlowModal,
} from '@modrinth/ui'
import { provide, ref, useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'
import { useRouter } from 'vue-router'

import type ModpackAlreadyInstalledModal from '@/components/ui/modal/ModpackAlreadyInstalledModal.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_search_results } from '@/helpers/cache.js'
import { import_instance } from '@/helpers/import.js'
import { create_profile_and_install, create_profile_and_install_from_file } from '@/helpers/pack'
import { create, list } from '@/helpers/profile.js'
import type { InstanceLoader } from '@/helpers/types'

export function setupCreationModal(notificationManager: AbstractWebNotificationManager) {
	const { handleError } = notificationManager
	const router = useRouter()

	const installationModal =
		useTemplateRef<ComponentExposed<typeof CreationFlowModal>>('installationModal')
	const modpackAlreadyInstalledModal = ref<InstanceType<typeof ModpackAlreadyInstalledModal>>()

	function setModpackAlreadyInstalledModal(
		modal: InstanceType<typeof ModpackAlreadyInstalledModal>,
	) {
		modpackAlreadyInstalledModal.value = modal
	}

	async function fetchExistingInstanceNames(): Promise<string[]> {
		const instances = await list().catch(handleError)
		return instances?.map((i) => i.name) ?? []
	}

	provide('showCreationModal', () => {
		installationModal.value?.show()
	})

	async function proceedWithModpackCreation(
		projectId: string,
		versionId: string,
		name: string,
		iconUrl?: string,
	) {
		await create_profile_and_install(projectId, versionId, name, iconUrl).catch(handleError)
		trackEvent('InstanceCreate', { source: 'CreationModalModpack' })
	}

	async function handleCreate(config: CreationFlowContextValue) {
		try {
			if (config.modpackSelection.value) {
				const { projectId, versionId, name, iconUrl } = config.modpackSelection.value

				const instances = await list().catch(handleError)
				const existingInstance = instances?.find((i) => i.linked_data?.project_id === projectId)

				if (existingInstance) {
					pendingModpackCreation.value = { projectId, versionId, name, iconUrl }
					installationModal.value?.hide()
					modpackAlreadyInstalledModal.value?.show(existingInstance.name, existingInstance.path)
					return
				}
			}

			installationModal.value?.hide()

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
				await proceedWithModpackCreation(projectId, versionId, name, iconUrl)
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
			const name = config.instanceName.value.trim() || config.autoInstanceName.value

			await create(
				name,
				config.selectedGameVersion.value!,
				loader as InstanceLoader,
				loaderVersion,
				iconPath,
				false,
			).catch(handleError)

			trackEvent('InstanceCreate', {
				source: 'CreationModal',
			})
		} catch (err) {
			handleError(err as Error)
		}
	}

	const pendingModpackCreation = ref<{
		projectId: string
		versionId: string
		name: string
		iconUrl?: string
	} | null>(null)

	async function handleModpackDuplicateCreateAnyway() {
		if (!pendingModpackCreation.value) return
		const { projectId, versionId, name, iconUrl } = pendingModpackCreation.value
		pendingModpackCreation.value = null
		await proceedWithModpackCreation(projectId, versionId, name, iconUrl)
	}

	function handleModpackDuplicateGoToInstance(instancePath: string) {
		pendingModpackCreation.value = null
		router.push(`/instance/${encodeURIComponent(instancePath)}/`)
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
		fetchExistingInstanceNames,
		handleCreate,
		handleBrowseModpacks,
		searchModpacks,
		getProjectVersions,
		setModpackAlreadyInstalledModal,
		handleModpackDuplicateCreateAnyway,
		handleModpackDuplicateGoToInstance,
	}
}
