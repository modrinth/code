<template>
	<IconEditorModal
		ref="iconEditorModal"
		:config="generatedConfig"
		:load-recents="loadRecentConfigs"
		:save="saveGeneratedIcon"
	/>
	<div class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">Icon</span>
		<div class="group relative w-fit">
			<TeleportOverflowMenu
				:label="editIconTooltip"
				:tooltip="editIconTooltip"
				:icon-only="false"
				type="quiet"
				interaction="none"
				class="m-0 !h-auto cursor-pointer appearance-none border-none bg-transparent !p-0 transition-transform group-active:scale-95"
				:disabled="isIconActionDisabled"
				:options="[
					{
						id: 'upload',
						label: 'Upload icon',
						action: () => triggerFileInput(),
						disabled: !props.canEdit,
						tooltip: !props.canEdit ? editIconTooltip : undefined,
					},
					{
						id: 'create',
						label: generatedConfig ? 'Edit created icon' : 'Create an icon',
						action: () => openIconEditor(),
						disabled: !props.canEdit,
						tooltip: !props.canEdit ? editIconTooltip : undefined,
					},
					{
						id: 'sync',
						label: 'Reset icon',
						action: () => resetIcon(),
						disabled: !props.canEdit,
						tooltip: !props.canEdit ? editIconTooltip : undefined,
					},
				]"
			>
				<ServerIcon
					class="size-28 transition-[filter] group-hover:brightness-[0.50]"
					:class="isIconActionLoading ? 'brightness-[0.50]' : ''"
					:image="displayIcon"
				/>
				<div
					class="absolute top-0 h-full w-full flex items-center justify-center"
					:class="isIconActionLoading ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
				>
					<SpinnerIcon
						v-if="isIconActionLoading"
						aria-hidden="true"
						class="h-10 w-10 animate-spin text-primary"
					/>
					<EditIcon v-else aria-hidden="true" class="h-10 w-10 text-primary" />
				</div>
				<template #upload> <UploadIcon /> Upload icon </template>
				<template #create> <PaletteIcon /> {{ generatedConfig ? 'Edit created icon' : 'Create an icon' }} </template>
				<template #sync> <TransferIcon /> Reset icon </template>
			</TeleportOverflowMenu>
		</div>
	</div>
</template>

<script setup lang="ts">
import { EditIcon, PaletteIcon, SpinnerIcon, TransferIcon, UploadIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, onMounted, ref } from 'vue'

import TeleportOverflowMenu from '#ui/components/base/buttons/TeleportOverflowMenu.vue'
import IconEditorModal from '#ui/components/base/icon-editor-modal/index.vue'
import { renderIcon, type IconConfig } from '#ui/components/base/icon-editor-modal'
import ServerIcon from '#ui/components/servers/icons/ServerIcon.vue'
import { processImageBlob, useServerImage } from '#ui/composables/use-server-image'
import { useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

const props = withDefaults(
	defineProps<{
		canEdit?: boolean
		permissionDeniedMessage?: string
	}>(),
	{
		canEdit: true,
		permissionDeniedMessage: undefined,
	},
)

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { serverId, server } = injectModrinthServerContext()
const queryClient = useQueryClient()
const iconEditorModal = ref<InstanceType<typeof IconEditorModal> | null>(null)
const generatedConfig = ref<IconConfig | null>(null)
const isUploadingIcon = ref(false)
const isSyncingIcon = ref(false)
const isIconActionLoading = computed(() => isUploadingIcon.value || isSyncingIcon.value)
const isIconActionDisabled = computed(() => isIconActionLoading.value || !props.canEdit)
const editIconTooltip = computed(() =>
	props.canEdit
		? 'Edit icon'
		: (props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction)),
)

const {
	image: displayIcon,
	refetch: refetchRemoteIcon,
	setImage,
	clearImage,
	resetLocalOverride,
} = useServerImage(
	serverId,
	computed(() => server.value?.upstream ?? null),
	{
		includeProjectFallback: false,
	},
)

function getStatusCode(error: unknown): number | undefined {
	const err = error as { statusCode?: number; response?: { status?: number } }
	return err.statusCode ?? err.response?.status
}

function isNotFound(error: unknown): boolean {
	return getStatusCode(error) === 404
}

const configPath = '/server-icon-config.json'
const recentsKey = 'modrinth.server-icon-recents'

async function loadRecentConfigs(): Promise<IconConfig[]> {
	try {
		const value: unknown = JSON.parse(localStorage.getItem(recentsKey) ?? '[]')
		return Array.isArray(value) ? (value as IconConfig[]) : []
	} catch {
		return []
	}
}

async function saveRecentConfig(config: IconConfig) {
	try {
		const recent = await loadRecentConfigs()
		const key = JSON.stringify(config)
		localStorage.setItem(
			recentsKey,
			JSON.stringify([config, ...recent.filter((entry) => JSON.stringify(entry) !== key)].slice(0, 16)),
		)
	} catch {
		return
	}
}

async function deleteFile(fsAuth: Awaited<ReturnType<typeof client.archon.servers_v0.getFilesystemAuth>>, path: string) {
	try {
		await client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, path, false)
	} catch (error) {
		if (!isNotFound(error)) throw error
	}
}

async function loadGeneratedConfig() {
	try {
		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const blob = await client.kyros.files_v0.downloadFileWithAuth(fsAuth, configPath)
		const config: unknown = JSON.parse(await blob.text())
		generatedConfig.value =
			config && typeof config === 'object' && 'symbol' in config &&
			typeof config.symbol === 'string' && 'background' in config
				? (config as IconConfig)
				: null
	} catch (error) {
		if (!isNotFound(error)) console.debug('Server icon config fetch failed:', error)
		generatedConfig.value = null
	}
}

onMounted(() => void loadGeneratedConfig())

async function openIconEditor() {
	if (isIconActionDisabled.value) return
	await loadGeneratedConfig()
	iconEditorModal.value?.show()
}

async function uploadIcon(file: File, config: IconConfig | null) {
	if (isIconActionDisabled.value) return
	isUploadingIcon.value = true
	try {
		const scaledFile = await new Promise<File>((resolve, reject) => {
			const canvas = document.createElement('canvas')
			const ctx = canvas.getContext('2d')
			if (!ctx) {
				reject(new Error('Could not resize the icon image.'))
				return
			}
			const img = new Image()
			const url = URL.createObjectURL(file)
			img.onload = () => {
				canvas.width = 64
				canvas.height = 64
				ctx.drawImage(img, 0, 0, 64, 64)
				canvas.toBlob((blob) => {
					URL.revokeObjectURL(url)
					if (blob) resolve(new File([blob], 'server-icon.png', { type: 'image/png' }))
					else reject(new Error('Could not resize the icon image.'))
				}, 'image/png')
			}
			img.onerror = () => {
				URL.revokeObjectURL(url)
				reject(new Error('Could not read the icon image.'))
			}
			img.src = url
		})

		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		try {
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon.png', scaledFile).promise
		} catch (uploadError) {
			await deleteFile(fsAuth, '/server-icon.png')
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon.png', scaledFile).promise
		}

		try {
			await deleteFile(fsAuth, '/server-icon-original.png')
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon-original.png', file)
				.promise
		} catch (error) {
			console.debug('Server icon original upload failed:', error)
		}
		await deleteFile(fsAuth, configPath)
		if (config) {
			const configFile = new File([JSON.stringify(config)], 'server-icon-config.json', {
				type: 'application/json',
			})
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, configPath, configFile).promise
		}
		generatedConfig.value = config
		if (config) await saveRecentConfig(config)

		const dataURL = await processImageBlob(file, 512)
		setImage(dataURL)
		queryClient.setQueriesData({ queryKey: ['servers', 'detail', serverId, 'icon'] }, dataURL)
		const remoteIcon = await refetchRemoteIcon()
		if (remoteIcon.data) resetLocalOverride()
		await queryClient.invalidateQueries({ queryKey: ['server-icon', serverId] })

		addNotification({
			type: 'success',
			title: 'Server icon updated',
			text: 'Your server icon was successfully changed.',
		})
	} finally {
		isUploadingIcon.value = false
	}
}

async function uploadFile(event: Event) {
	const file = (event.target as HTMLInputElement).files?.[0]
	if (!file || isIconActionDisabled.value) return
	try {
		await uploadIcon(file, null)
	} catch {
		addNotification({ type: 'error', title: 'Upload failed', text: 'Failed to upload server icon.' })
	}
}

async function saveGeneratedIcon(config: IconConfig, symbolAsset: string) {
	if (isIconActionDisabled.value) throw new Error('Server icon editing is unavailable.')
	const icon = await renderIcon(config, symbolAsset)
	await uploadIcon(icon, config)
}

const resetIcon = async () => {
	if (isIconActionDisabled.value) return
	isSyncingIcon.value = true

	try {
		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const deleteResults = await Promise.allSettled([
			client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, '/server-icon.png', false),
			client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, '/server-icon-original.png', false),
			client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, configPath, false),
		])

		for (const result of deleteResults) {
			if (result.status === 'rejected' && !isNotFound(result.reason)) {
				throw result.reason
			}
		}

		// Force default icon state across all useServerImage instances via the shared query cache.
		// Use `null` (not `undefined`) because TanStack Query v5 treats setQueriesData(undefined)
		// as a no-op. The `null` sentinel is handled by useServerImage's image computed.
		generatedConfig.value = null
		clearImage()
		await queryClient.cancelQueries({ queryKey: ['servers', 'detail', serverId, 'icon'] })
		queryClient.setQueriesData({ queryKey: ['servers', 'detail', serverId, 'icon'] }, null)
		await queryClient.invalidateQueries({ queryKey: ['server-icon', serverId] })

		addNotification({
			type: 'success',
			title: 'Server icon reset',
			text: 'Your server icon was successfully reset.',
		})
	} catch {
		addNotification({
			type: 'error',
			title: 'Reset failed',
			text: 'Failed to reset server icon.',
		})
	} finally {
		isSyncingIcon.value = false
	}
}

const triggerFileInput = () => {
	if (isIconActionDisabled.value) return

	const input = document.createElement('input')
	input.type = 'file'
	input.id = 'server-icon-field'
	input.accept = 'image/png,image/jpeg,image/gif,image/webp'
	const cleanup = () => {
		input.remove()
		window.removeEventListener('focus', handleWindowFocus)
	}
	const handleWindowFocus = () => {
		// If picker was cancelled there is no change event; clean up on focus return.
		setTimeout(() => {
			if (!input.value) cleanup()
		}, 0)
	}
	input.onchange = async (event) => {
		try {
			await uploadFile(event)
		} finally {
			cleanup()
		}
	}
	document.body.appendChild(input)
	window.addEventListener('focus', handleWindowFocus, { once: true })
	input.click()
}
</script>
