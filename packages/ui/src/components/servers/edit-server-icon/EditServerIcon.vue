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
				<template #create>
					<PaletteIcon /> {{ generatedConfig ? 'Edit created icon' : 'Create an icon' }}
				</template>
				<template #sync> <TransferIcon /> Reset icon </template>
			</TeleportOverflowMenu>
		</div>
	</div>
</template>

<script setup lang="ts">
import { EditIcon, PaletteIcon, SpinnerIcon, TransferIcon, UploadIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import TeleportOverflowMenu from '#ui/components/base/buttons/TeleportOverflowMenu.vue'
import { type IconConfig, renderIcon } from '#ui/components/base/icon-editor-modal'
import IconEditorModal from '#ui/components/base/icon-editor-modal/index.vue'
import ServerIcon from '#ui/components/servers/icons/ServerIcon.vue'
import { useVIntl } from '#ui/composables/i18n'
import { useServerImage } from '#ui/composables/use-server-image'
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
const { serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()
const iconEditorModal = ref<InstanceType<typeof IconEditorModal> | null>(null)
const isIconActionLoading = computed(
	() => uploadMutation.isPending.value || resetMutation.isPending.value,
)
const isIconActionDisabled = computed(() => isIconActionLoading.value || !props.canEdit)
const editIconTooltip = computed(() =>
	props.canEdit
		? 'Edit icon'
		: (props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction)),
)

const { image: displayIcon, queryKey: iconQueryKey } = useServerImage(serverId)

function getStatusCode(error: unknown): number | undefined {
	const err = error as { statusCode?: number; response?: { status?: number } }
	return err.statusCode ?? err.response?.status
}

function isNotFound(error: unknown): boolean {
	return getStatusCode(error) === 404
}

const configPath = '/server-icon-config.json'
const recentsKey = 'modrinth.server-icon-recents'
const configQueryKey = ['server-icon-config', serverId] as const
const configQuery = useQuery({
	queryKey: configQueryKey,
	queryFn: loadGeneratedConfig,
	enabled: typeof window !== 'undefined',
})
const generatedConfig = computed(() => configQuery.data.value ?? null)

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
			JSON.stringify(
				[config, ...recent.filter((entry) => JSON.stringify(entry) !== key)].slice(0, 16),
			),
		)
	} catch {
		return
	}
}

async function deleteFile(
	fsAuth: Awaited<ReturnType<typeof client.archon.servers_v0.getFilesystemAuth>>,
	path: string,
) {
	try {
		await client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, path, false)
	} catch (error) {
		if (!isNotFound(error)) throw error
	}
}

async function loadGeneratedConfig(): Promise<IconConfig | null> {
	try {
		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const blob = await client.kyros.files_v0.downloadFileWithAuth(fsAuth, configPath)
		const config: unknown = JSON.parse(await blob.text())
		return config &&
			typeof config === 'object' &&
			'symbol' in config &&
			typeof config.symbol === 'string' &&
			'background' in config
				? (config as IconConfig)
				: null
	} catch (error) {
		if (isNotFound(error)) return null
		throw error
	}
}

async function openIconEditor() {
	if (isIconActionDisabled.value) return
	await configQuery.refetch()
	iconEditorModal.value?.show()
}

const uploadMutation = useMutation({
	mutationFn: async ({ file, config }: { file: File; config: IconConfig | null }) => {
		await client.archon.icons_v1.set(serverId, file)
		let configFailed = false
		try {
			await queryClient.cancelQueries({ queryKey: configQueryKey })
			const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
			await deleteFile(fsAuth, configPath)
			if (config) {
				const configFile = new File([JSON.stringify(config)], 'server-icon-config.json', {
					type: 'application/json',
				})
				await client.kyros.files_v0.uploadFileWithAuth(fsAuth, configPath, configFile).promise
			}
			queryClient.setQueryData(configQueryKey, config)
			if (config) await saveRecentConfig(config)
		} catch {
			configFailed = true
		}

		return configFailed
	},
	onSuccess: async (configFailed) => {
		await queryClient.cancelQueries({ queryKey: iconQueryKey.value })
		await queryClient.invalidateQueries({ queryKey: iconQueryKey.value })
		if (configFailed) await configQuery.refetch()
		addNotification({
			type: configFailed ? 'error' : 'success',
			title: configFailed ? 'Icon editor settings not saved' : 'Server icon updated',
			text: configFailed
				? 'The server icon was updated, but its editor settings could not be saved.'
				: 'Your server icon was successfully changed.',
		})
	},
})

async function uploadIcon(file: File, config: IconConfig | null) {
	if (isIconActionDisabled.value) return
	await uploadMutation.mutateAsync({ file, config })
}

async function uploadFile(event: Event) {
	const file = (event.target as HTMLInputElement).files?.[0]
	if (!file || isIconActionDisabled.value) return
	try {
		await uploadIcon(file, null)
	} catch {
		addNotification({
			type: 'error',
			title: 'Upload failed',
			text: 'Failed to upload server icon.',
		})
	}
}

async function saveGeneratedIcon(config: IconConfig, symbolAsset: string) {
	if (isIconActionDisabled.value) throw new Error('Server icon editing is unavailable.')
	const icon = await renderIcon(config, symbolAsset)
	await uploadIcon(icon, config)
}

const resetMutation = useMutation({
	mutationFn: async () => {
		try {
			await client.archon.icons_v1.delete(serverId)
		} catch (error) {
			if (!isNotFound(error)) throw error
		}
		try {
			await queryClient.cancelQueries({ queryKey: configQueryKey })
			const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
			await deleteFile(fsAuth, configPath)
			queryClient.setQueryData(configQueryKey, null)
			return false
		} catch {
			return true
		}
	},
	onSuccess: async (configFailed) => {
		await queryClient.cancelQueries({ queryKey: iconQueryKey.value })
		queryClient.setQueryData(iconQueryKey.value, null)
		if (configFailed) await configQuery.refetch()
		addNotification({
			type: configFailed ? 'error' : 'success',
			title: configFailed ? 'Icon editor settings not cleared' : 'Server icon reset',
			text: configFailed
				? 'The server icon was reset, but its editor settings could not be cleared.'
				: 'Your server icon was successfully reset.',
		})
	},
})

const resetIcon = async () => {
	if (isIconActionDisabled.value) return
	try {
		await resetMutation.mutateAsync()
	} catch {
		addNotification({
			type: 'error',
			title: 'Reset failed',
			text: 'Failed to reset server icon.',
		})
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
