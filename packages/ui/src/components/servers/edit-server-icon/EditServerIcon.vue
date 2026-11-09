<template>
	<IconEditorModal
		ref="iconEditorModal"
		:config="editorConfig"
		:load-recents="loadRecentConfigs"
		:save="saveGeneratedIcon"
	/>
	<div class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">Icon</span>
		<div class="flex items-center gap-4 sm:hidden">
			<div class="relative w-fit">
				<ServerIcon
					class="size-28"
					:class="isIconActionLoading ? 'brightness-[0.50]' : ''"
					:image="image"
					:config="config"
				/>
				<div
					v-if="isIconActionLoading"
					class="absolute top-0 h-full w-full flex items-center justify-center"
				>
					<SpinnerIcon aria-hidden="true" class="h-10 w-10 animate-spin text-primary" />
				</div>
			</div>
			<div
				v-tooltip="props.canEdit ? undefined : editIconTooltip"
				class="flex min-w-0 flex-col gap-2"
			>
				<Button :disabled="isIconActionDisabled" @click="triggerFileInput">
					<UploadIcon /> Upload icon
				</Button>
				<Button :disabled="isIconActionDisabled" @click="openIconEditor">
					<PaletteIcon /> {{ editorConfig ? 'Edit created icon' : 'Create an icon' }}
				</Button>
				<Button :disabled="isIconActionDisabled" @click="resetIcon">
					<TransferIcon /> Reset icon
				</Button>
			</div>
		</div>
		<div class="group relative hidden w-fit sm:block">
			<TeleportOverflowMenu
				:label="editIconTooltip"
				:tooltip="props.canEdit ? undefined : editIconTooltip"
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
						label: editorConfig ? 'Edit created icon' : 'Create an icon',
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
					:image="image"
					:config="config"
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
					<PaletteIcon /> {{ editorConfig ? 'Edit created icon' : 'Create an icon' }}
				</template>
				<template #sync> <TransferIcon /> Reset icon </template>
			</TeleportOverflowMenu>
		</div>
	</div>
</template>

<script setup lang="ts">
import { type Archon, ModrinthApiError } from '@modrinth/api-client'
import { EditIcon, PaletteIcon, SpinnerIcon, TransferIcon, UploadIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import TeleportOverflowMenu from '#ui/components/base/buttons/TeleportOverflowMenu.vue'
import { type IconConfig, renderIcon } from '#ui/components/base/icon-editor-modal'
import IconEditorModal from '#ui/components/base/icon-editor-modal/index.vue'
import ServerIcon from '#ui/components/servers/icons/ServerIcon.vue'
import { serverIconKeys, useServerIcon } from '#ui/composables'
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
const { serverId } = injectModrinthServerContext()
const queryClient = useQueryClient()
const iconEditorModal = ref<InstanceType<typeof IconEditorModal> | null>(null)
const isUploadingIcon = ref(false)
const isSyncingIcon = ref(false)
const isIconActionLoading = computed(() => isUploadingIcon.value || isSyncingIcon.value)
const isIconActionDisabled = computed(() => isIconActionLoading.value || !props.canEdit)
const editIconTooltip = computed(() =>
	props.canEdit
		? 'Edit icon'
		: (props.permissionDeniedMessage ?? formatMessage(commonMessages.noPermissionAction)),
)

const { metadata, config, image } = useServerIcon(serverId)
const editorConfig = computed(() => metadata.value?.config ?? null)

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
			JSON.stringify(
				[config, ...recent.filter((entry) => JSON.stringify(entry) !== key)].slice(0, 16),
			),
		)
	} catch {
		return
	}
}

async function setServerIcon(icon: Archon.Icons.v1.ServerIcon | null) {
	await queryClient.cancelQueries({ queryKey: serverIconKeys.metadata(serverId), exact: true })
	queryClient.setQueryData(serverIconKeys.metadata(serverId), icon)
}

function openIconEditor() {
	if (isIconActionDisabled.value) return
	iconEditorModal.value?.show()
}

async function saveGeneratedIcon(config: IconConfig, symbolAsset: string) {
	if (isIconActionDisabled.value) throw new Error('Server icon editing is unavailable.')
	const icon = await renderIcon(config, symbolAsset)
	if (!(await uploadIcon(icon, config))) throw new Error('Failed to upload server icon.')
	await saveRecentConfig(config)
}

const uploadFile = async (e: Event) => {
	if (isIconActionDisabled.value) return

	const file = (e.target as HTMLInputElement).files?.[0]
	if (!file) {
		addNotification({
			type: 'error',
			title: 'No file selected',
			text: 'Please select a file to upload.',
		})
		return
	}

	await uploadIcon(file, null)
}

async function uploadIcon(file: File, config: IconConfig | null): Promise<boolean> {
	isUploadingIcon.value = true

	try {
		const icon = config
			? await client.archon.icons_v1.setGenerated(serverId, file, config)
			: await client.archon.icons_v1.set(serverId, file)
		await setServerIcon(icon)

		addNotification({
			type: 'success',
			title: 'Server icon updated',
			text: 'Your server icon was successfully changed.',
		})
		return true
	} catch {
		addNotification({
			type: 'error',
			title: 'Upload failed',
			text: 'Failed to upload server icon.',
		})
		return false
	} finally {
		isUploadingIcon.value = false
	}
}

const resetIcon = async () => {
	if (isIconActionDisabled.value) return
	isSyncingIcon.value = true

	try {
		try {
			await client.archon.icons_v1.delete(serverId)
		} catch (error) {
			if (!(error instanceof ModrinthApiError && error.statusCode === 404)) throw error
		}
		await setServerIcon(null)

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
