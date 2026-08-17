<script setup lang="ts">
import {
	CopyIcon,
	EditIcon,
	PaletteIcon,
	SpinnerIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	Chips,
	defineMessages,
	injectNotificationManager,
	StyledInput,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, type Ref, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import IconEditorModal from '@/components/ui/instance_settings/icon-editor-modal/index.vue'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { trackEvent } from '@/helpers/analytics'
import { install_duplicate_instance } from '@/helpers/install'
import { edit, edit_icon, getInstanceIconUrl, remove } from '@/helpers/instance'
import type { GameInstance, InstanceIconConfig } from '@/helpers/types'

import { injectInstanceSettings } from './instance-settings-context'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()
const queryClient = useQueryClient()

const deleteConfirmModal = ref()
const iconEditorModal = ref<InstanceType<typeof IconEditorModal> | null>(null)

const { instance } = injectInstanceSettings()
type ReleaseChannel = GameInstance['update_channel']
const releaseChannelOptions: ReleaseChannel[] = ['release', 'beta', 'alpha']

const title = ref(instance.value.name)
const icon: Ref<string | undefined> = ref(instance.value.icon_path)
const iconConfig = ref<InstanceIconConfig | null>(instance.value.icon_config ?? null)
const savingReleaseChannel = ref(false)
const selectedReleaseChannel = ref<ReleaseChannel>(instance.value.update_channel)
const releaseChannelDisabledItems = computed<ReleaseChannel[]>(() =>
	savingReleaseChannel.value ? [...releaseChannelOptions] : [],
)

const installing = computed(() => instance.value.install_stage !== 'installed')

watch(
	() => [instance.value.id, instance.value.icon_path, instance.value.icon_config] as const,
	() => {
		icon.value = instance.value.icon_path
		iconConfig.value = instance.value.icon_config ?? null
	},
)

async function duplicateInstance() {
	await install_duplicate_instance(instance.value.id).catch(handleError)
	trackEvent('InstanceDuplicate', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
	})
}

function formatReleaseChannelLabel(channel: ReleaseChannel) {
	switch (channel) {
		case 'release':
			return formatMessage(messages.updateChannelRelease)
		case 'beta':
			return formatMessage(messages.updateChannelBeta)
		case 'alpha':
			return formatMessage(messages.updateChannelAlpha)
	}
}

function formatReleaseChannelDescription(channel: ReleaseChannel) {
	switch (channel) {
		case 'release':
			return formatMessage(messages.updateChannelReleaseDescription)
		case 'beta':
			return formatMessage(messages.updateChannelBetaDescription)
		case 'alpha':
			return formatMessage(messages.updateChannelAlphaDescription)
	}
}

watch(
	() => [instance.value.id, instance.value.update_channel] as const,
	() => {
		if (!savingReleaseChannel.value) {
			selectedReleaseChannel.value = instance.value.update_channel
		}
	},
)

watch(selectedReleaseChannel, async (channel, previousChannel) => {
	const previousReleaseChannel = previousChannel ?? instance.value.update_channel
	if (channel === instance.value.update_channel) return

	savingReleaseChannel.value = true
	const instanceId = instance.value.id
	await edit(instanceId, { update_channel: channel })
		.then(() => queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instanceId] }))
		.catch((error) => {
			selectedReleaseChannel.value = previousReleaseChannel
			handleError(error)
		})
	savingReleaseChannel.value = false
})

async function resetIcon() {
	try {
		await edit_icon(instance.value.id, null)
		icon.value = undefined
		iconConfig.value = null
	} catch (error) {
		handleError(error)
		return
	}
	trackEvent('InstanceRemoveIcon')
}

async function setIcon() {
	const value = await open({
		multiple: false,
		filters: [
			{
				name: 'Image',
				extensions: ['png', 'jpeg', 'svg', 'webp', 'gif', 'jpg'],
			},
		],
	})

	if (!value) return

	try {
		await edit_icon(instance.value.id, value)
		icon.value = value
		iconConfig.value = null
	} catch (error) {
		handleError(error)
		return
	}

	trackEvent('InstanceSetIcon')
}

function openIconEditor() {
	iconEditorModal.value?.show()
	trackEvent(iconConfig.value ? 'InstanceEditCreatedIcon' : 'InstanceCreateIcon')
}

function onGeneratedIconSaved(iconPath: string, config: InstanceIconConfig) {
	icon.value = iconPath
	iconConfig.value = config
	trackEvent('InstanceSaveCreatedIcon')
}

const editInstanceObject = computed(() => ({
	name: title.value.trim().substring(0, 80) ?? 'Instance',
}))

watch(
	title,
	async () => {
		if (removing.value) return
		await edit(instance.value.id, editInstanceObject.value).catch(handleError)
	},
	{ deep: true },
)

const removing = ref(false)
async function removeInstance() {
	removing.value = true
	const path = instance.value.id

	trackEvent('InstanceRemove', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
	})

	await router.push({ path: '/' })
	await remove(path).catch(handleError)
}

const messages = defineMessages({
	name: {
		id: 'instance.settings.tabs.general.name',
		defaultMessage: 'Name',
	},
	icon: {
		id: 'instance.settings.tabs.general.icon',
		defaultMessage: 'Icon',
	},
	editIcon: {
		id: 'instance.settings.tabs.general.edit-icon',
		defaultMessage: 'Edit icon',
	},
	selectIcon: {
		id: 'instance.settings.tabs.general.edit-icon.select',
		defaultMessage: 'Select icon',
	},
	replaceIcon: {
		id: 'instance.settings.tabs.general.edit-icon.replace',
		defaultMessage: 'Replace icon',
	},
	createIcon: {
		id: 'instance.settings.tabs.general.edit-icon.create',
		defaultMessage: 'Create an icon',
	},
	editCreatedIcon: {
		id: 'instance.settings.tabs.general.edit-icon.edit-created',
		defaultMessage: 'Edit icon',
	},
	removeIcon: {
		id: 'instance.settings.tabs.general.edit-icon.remove',
		defaultMessage: 'Remove icon',
	},
	duplicateInstance: {
		id: 'instance.settings.tabs.general.duplicate-instance',
		defaultMessage: 'Duplicate instance',
	},
	duplicateInstanceDescription: {
		id: 'instance.settings.tabs.general.duplicate-instance.description',
		defaultMessage: 'Creates a copy of this instance, including worlds, configs, mods, etc.',
	},
	duplicateButtonTooltipInstalling: {
		id: 'instance.settings.tabs.general.duplicate-button.tooltip.installing',
		defaultMessage: 'Cannot duplicate while installing.',
	},
	duplicateButton: {
		id: 'instance.settings.tabs.general.duplicate-button',
		defaultMessage: 'Duplicate',
	},
	updateChannel: {
		id: 'instance.settings.tabs.general.update-channel',
		defaultMessage: 'Update channel',
	},
	updateChannelReleaseDescription: {
		id: 'instance.settings.tabs.general.update-channel.release.description',
		defaultMessage: 'Only release versions will be shown as available updates.',
	},
	updateChannelBetaDescription: {
		id: 'instance.settings.tabs.general.update-channel.beta.description',
		defaultMessage: 'Release and beta versions will be shown as available updates.',
	},
	updateChannelAlphaDescription: {
		id: 'instance.settings.tabs.general.update-channel.alpha.description',
		defaultMessage: 'Release, beta, and alpha versions will be shown as available updates.',
	},
	updateChannelRelease: {
		id: 'instance.settings.tabs.general.update-channel.release',
		defaultMessage: 'Release',
	},
	updateChannelBeta: {
		id: 'instance.settings.tabs.general.update-channel.beta',
		defaultMessage: 'Beta',
	},
	updateChannelAlpha: {
		id: 'instance.settings.tabs.general.update-channel.alpha',
		defaultMessage: 'Alpha',
	},
	selectUpdateChannelAriaLabel: {
		id: 'instance.settings.tabs.general.update-channel.select',
		defaultMessage: 'Select update channel',
	},
	deleteInstance: {
		id: 'instance.settings.tabs.general.delete',
		defaultMessage: 'Delete instance',
	},
	deleteInstanceDescription: {
		id: 'instance.settings.tabs.general.delete.description',
		defaultMessage:
			'Permanently deletes an instance from your device, including your worlds, configs, and all installed content. Be careful, as once you delete a instance there is no way to recover it.',
	},
	deleteInstanceButton: {
		id: 'instance.settings.tabs.general.delete.button',
		defaultMessage: 'Delete instance',
	},
	deletingInstanceButton: {
		id: 'instance.settings.tabs.general.deleting.button',
		defaultMessage: 'Deleting...',
	},
})
</script>

<template>
	<ConfirmDeleteInstanceModal
		ref="deleteConfirmModal"
		:instances="[instance]"
		@delete="removeInstance"
	/>
	<IconEditorModal
		ref="iconEditorModal"
		:instance-id="instance.id"
		:config="iconConfig"
		@saved="onGeneratedIconSaved"
	/>
	<div class="block">
		<div class="float-end ml-10 relative group w-fit">
			<div class="flex flex-col gap-1">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.icon) }}
				</span>
				<div class="group relative w-fit">
					<TeleportOverflowMenu
						:label="formatMessage(messages.editIcon)"
						:tooltip="formatMessage(messages.editIcon)"
						:icon-only="false"
						type="quiet"
						interaction="none"
						class="m-0 !h-auto cursor-pointer appearance-none border-none bg-transparent !p-0 transition-transform group-active:scale-95"
						:options="[
							{
								id: 'select',
								label: icon
									? formatMessage(messages.replaceIcon)
									: formatMessage(messages.selectIcon),
								action: () => setIcon(),
							},
							{
								id: 'create',
								action: () => openIconEditor(),
							},
							{
								id: 'remove',
								label: formatMessage(messages.removeIcon),
								action: () => resetIcon(),
								shown: !!icon,
							},
						]"
					>
						<Avatar
							:src="getInstanceIconUrl(icon)"
							size="108px"
							class="transition-[filter] group-hover:brightness-75"
							:tint-by="instance.id"
							no-shadow
						/>
						<div
							class="absolute top-0 h-full w-full flex items-center justify-center opacity-0 transition-all group-hover:opacity-100"
						>
							<EditIcon aria-hidden="true" class="h-10 w-10 text-white opacity-70" />
						</div>
						<template #select>
							<UploadIcon />
							{{ icon ? formatMessage(messages.replaceIcon) : formatMessage(messages.selectIcon) }}
						</template>
						<template #create>
							<PaletteIcon />
							{{ formatMessage(iconConfig ? messages.editCreatedIcon : messages.createIcon) }}
						</template>
						<template #remove> <TrashIcon /> {{ formatMessage(messages.removeIcon) }} </template>
					</TeleportOverflowMenu>
				</div>
			</div>
		</div>
		<label for="instance-name" class="m-0 text-lg font-semibold text-contrast block">
			{{ formatMessage(messages.name) }}
		</label>
		<div class="flex">
			<StyledInput
				id="instance-name"
				v-model="title"
				autocomplete="off"
				:maxlength="80"
				wrapper-class="flex-grow"
			/>
		</div>
		<template v-if="instance.install_stage == 'installed'">
			<div class="flex flex-col gap-2.5 mt-6">
				<h2 id="duplicate-instance-label" class="m-0 text-lg font-semibold text-contrast block">
					{{ formatMessage(messages.duplicateInstance) }}
				</h2>
				<Button
					v-tooltip="installing ? formatMessage(messages.duplicateButtonTooltipInstalling) : null"
					aria-labelledby="duplicate-instance-label"
					:disabled="installing"
					class="w-max"
					@click="duplicateInstance"
				>
					<CopyIcon /> {{ formatMessage(messages.duplicateButton) }}
				</Button>
				<p class="m-0">
					{{ formatMessage(messages.duplicateInstanceDescription) }}
				</p>
			</div>
		</template>
		<div class="flex flex-col gap-2.5 mt-6">
			<h2 class="m-0 text-lg font-semibold text-contrast block">
				{{ formatMessage(messages.updateChannel) }}
			</h2>
			<Chips
				v-model="selectedReleaseChannel"
				:items="releaseChannelOptions"
				:format-label="formatReleaseChannelLabel"
				:capitalize="false"
				:disabled-items="releaseChannelDisabledItems"
				:aria-label="formatMessage(messages.selectUpdateChannelAriaLabel)"
			/>
			<p class="m-0">
				{{ formatReleaseChannelDescription(selectedReleaseChannel) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5 mt-6">
			<h2 id="delete-instance-label" class="m-0 text-lg font-semibold text-contrast block">
				{{ formatMessage(messages.deleteInstance) }}
			</h2>
			<Button
				type="colored"
				color="red"
				aria-labelledby="delete-instance-label"
				:disabled="removing"
				class="w-fit"
				@click="deleteConfirmModal.show()"
			>
				<SpinnerIcon v-if="removing" class="animate-spin" />
				<TrashIcon v-else />
				{{
					removing
						? formatMessage(messages.deletingInstanceButton)
						: formatMessage(messages.deleteInstanceButton)
				}}
			</Button>
			<p class="m-0">
				{{ formatMessage(messages.deleteInstanceDescription) }}
			</p>
		</div>
	</div>
</template>
<style scoped lang="scss">
.hovering-icon-shadow {
	box-shadow: var(--shadow-inset-sm), var(--shadow-raised);
}
</style>
