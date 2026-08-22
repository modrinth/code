import type {
	AbstractWebNotificationManager,
	CreationFlowContextValue,
	CreationFlowModal,
} from '@modrinth/ui'
import { provide, ref, useTemplateRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'
import { useRouter } from 'vue-router'

import type UnknownPackWarningModal from '@/components/ui/install_flow/UnknownPackWarningModal.vue'
import type ModpackAlreadyInstalledModal from '@/components/ui/modal/ModpackAlreadyInstalledModal.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { trackEvent } from '@/helpers/analytics'
import { get_search_results } from '@/helpers/cache.js'
import { import_instance } from '@/helpers/import.js'
import {
	type CreatePackLocation,
	install_create_instance,
	install_create_modpack_instance,
	install_get_modpack_preview,
	installJobInstanceId,
} from '@/helpers/install'
import { list } from '@/helpers/instance'
import { get_loader_versions as getLoaderManifest } from '@/helpers/metadata.js'
import type { InstanceIconConfig, InstanceLoader } from '@/helpers/types'

export function setupCreationModal(
	notificationManager: AbstractWebNotificationManager,
	getGeneratedIconConfig?: (iconPath: string) => InstanceIconConfig | null,
) {
	const { handleError } = notificationManager
	const router = useRouter()
	const appSettings = useAppSettings()

	const installationModal =
		useTemplateRef<ComponentExposed<typeof CreationFlowModal>>('installationModal')
	const unknownPackWarningModal =
		useTemplateRef<InstanceType<typeof UnknownPackWarningModal>>('unknownPackWarningModal')
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
	provide('showImportModal', async () => {
		await installationModal.value?.show()
		installationModal.value?.ctx.setImportMode()
	})

	async function navigateToCreatedInstance(
		job: Awaited<ReturnType<typeof install_create_instance>>,
	) {
		const instanceId = installJobInstanceId(job)
		if (instanceId) {
			await router.push(`/instance/${encodeURIComponent(instanceId)}`)
		}
	}

	async function proceedWithModpackCreation(
		projectId: string,
		versionId: string,
		name: string,
		iconUrl?: string,
	) {
		const job = await install_create_modpack_instance({
			type: 'fromVersionId',
			project_id: projectId,
			version_id: versionId,
			title: name,
			icon_url: iconUrl,
		})
		await navigateToCreatedInstance(job)
		trackEvent('InstanceCreate', { source: 'CreationModalModpack' })
	}

	async function proceedWithModpackFileCreation(location: CreatePackLocation) {
		const job = await install_create_modpack_instance(location)
		await navigateToCreatedInstance(job)
		trackEvent('InstanceCreate', { source: 'CreationModalModpackFile' })
	}

	async function handleCreate(config: CreationFlowContextValue) {
		try {
			if (config.modpackSelection.value) {
				const { projectId, versionId, name, iconUrl } = config.modpackSelection.value

				const instances = await list().catch(handleError)
				const existingInstance = instances?.find((i) => i.link?.project_id === projectId)

				if (existingInstance && !appSettings.getFeatureFlag('skip_non_essential_warnings')) {
					pendingModpackCreation.value = { projectId, versionId, name, iconUrl }
					installationModal.value?.hide()
					modpackAlreadyInstalledModal.value?.show(existingInstance.name, existingInstance.id)
					return
				}
			}

			installationModal.value?.hide()

			if (config.isImportMode.value) {
				let importCount = 0
				let importedInstanceId: string | null = null
				for (const [launcherName, instanceSet] of Object.entries(
					config.importSelectedInstances.value,
				)) {
					const launcher = config.importLaunchers.value.find((l) => l.name === launcherName)
					if (!launcher || instanceSet.size === 0) continue
					for (const name of instanceSet) {
						importCount += 1
						const job = await import_instance(launcher.name, launcher.path, name).catch(handleError)
						if (job) {
							importedInstanceId = installJobInstanceId(job)
						}
					}
				}
				trackEvent('InstanceCreate', { source: 'CreationModalImport' })
				if (importCount === 1 && importedInstanceId) {
					await router.push(`/instance/${encodeURIComponent(importedInstanceId)}`)
				}
				return
			}

			if (config.modpackSelection.value) {
				const { projectId, versionId, name, iconUrl } = config.modpackSelection.value
				await proceedWithModpackCreation(projectId, versionId, name, iconUrl)
				return
			}

			if (config.modpackFilePath.value) {
				const location: CreatePackLocation = {
					type: 'fromFile',
					path: config.modpackFilePath.value,
				}
				const preview = await install_get_modpack_preview(location)

				if (preview.unknownFile || preview.externalFilesInModpack.length > 0) {
					const splitPath = config.modpackFilePath.value.split(/[\\/]/)
					const fileName = splitPath
						? splitPath[splitPath.length - 1]
						: config.modpackFilePath.value
					if (unknownPackWarningModal.value) {
						unknownPackWarningModal.value?.show(
							() => proceedWithModpackFileCreation(location).catch(handleError),
							fileName,
							preview.externalFilesInModpack,
						)
					} else {
						await proceedWithModpackFileCreation(location)
					}
				} else {
					await proceedWithModpackFileCreation(location)
				}
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

			const job = await install_create_instance({
				name,
				gameVersion: config.selectedGameVersion.value!,
				loader: loader as InstanceLoader,
				loaderVersion,
				iconPath,
				iconConfig: iconPath ? getGeneratedIconConfig?.(iconPath) : null,
			})
			await navigateToCreatedInstance(job)

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
		try {
			await proceedWithModpackCreation(projectId, versionId, name, iconUrl)
		} catch (error) {
			handleError(error as Error)
		}
	}

	function handleModpackDuplicateGoToInstance(instanceId: string) {
		pendingModpackCreation.value = null
		router.push(`/instance/${encodeURIComponent(instanceId)}/`)
	}

	function handleBrowseModpacks() {
		installationModal.value?.hide()
		router.push('/browse/modpack')
	}

	async function searchProjects(query: string, limit: number = 10) {
		const projectTypes = ['mod', 'modpack', 'resourcepack', 'shader', 'datapack']
		const facets = JSON.stringify([projectTypes.map((type) => `project_type:${type}`)])
		const params = [`facets=${encodeURIComponent(facets)}`, `limit=${limit}`]
		if (query) {
			params.push(`query=${encodeURIComponent(query)}`)
		}
		const raw = await get_search_results(`?${params.join('&')}`)
		if (raw?.result) return raw.result
		return { hits: [], offset: 0, limit, total_hits: 0 }
	}

	return {
		installationModal,
		unknownPackWarningModal,
		fetchExistingInstanceNames,
		handleCreate,
		handleBrowseModpacks,
		searchProjects,
		getLoaderManifest,
		setModpackAlreadyInstalledModal,
		handleModpackDuplicateCreateAnyway,
		handleModpackDuplicateGoToInstance,
	}
}
