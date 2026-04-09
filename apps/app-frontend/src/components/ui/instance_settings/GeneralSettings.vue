<script setup lang="ts">
import { CopyIcon, EditIcon, PlusIcon, SpinnerIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	defineMessages,
	injectNotificationManager,
	OverflowMenu,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, type Ref, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { trackEvent } from '@/helpers/analytics'
import { duplicate, edit, edit_icon, list, remove } from '@/helpers/profile'
import { injectInstanceSettings } from '@/providers/instance-settings'

import type { GameInstance } from '../../../helpers/types'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()

const deleteConfirmModal = ref()

const { instance } = injectInstanceSettings()

const title = ref(instance.value.name)
const icon: Ref<string | undefined> = ref(instance.value.icon_path)
const groups = ref([...instance.value.groups])

const newCategoryInput = ref('')

const installing = computed(() => instance.value.install_stage !== 'installed')

async function duplicateProfile() {
	await duplicate(instance.value.path).catch(handleError)
	trackEvent('InstanceDuplicate', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
	})
}

const allInstances = ref((await list()) as GameInstance[])
const availableGroups = computed(() => [
	...new Set([...allInstances.value.flatMap((instance) => instance.groups), ...groups.value]),
])

async function resetIcon() {
	icon.value = undefined
	await edit_icon(instance.value.path, null).catch(handleError)
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

	icon.value = value
	await edit_icon(instance.value.path, icon.value).catch(handleError)

	trackEvent('InstanceSetIcon')
}

const editProfileObject = computed(() => ({
	name: title.value.trim().substring(0, 32) ?? 'Instance',
	groups: groups.value.map((x) => x.trim().substring(0, 32)).filter((x) => x.length > 0),
}))

const toggleGroup = (group: string) => {
	if (groups.value.includes(group)) {
		groups.value = groups.value.filter((x) => x !== group)
	} else {
		groups.value.push(group)
	}
}

const addCategory = () => {
	const text = newCategoryInput.value.trim()

	if (text.length > 0) {
		groups.value.push(text.substring(0, 32))
		newCategoryInput.value = ''
	}
}

watch(
	[title, groups, groups],
	async () => {
		if (removing.value) return
		await edit(instance.value.path, editProfileObject.value).catch(handleError)
	},
	{ deep: true },
)

const removing = ref(false)
async function removeProfile() {
	removing.value = true
	const path = instance.value.path

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
	libraryGroups: {
		id: 'instance.settings.tabs.general.library-groups',
		defaultMessage: 'Library groups',
	},
	libraryGroupsDescription: {
		id: 'instance.settings.tabs.general.library-groups.description',
		defaultMessage:
			'Library groups allow you to organize your instances into different sections in your library.',
	},
	libraryGroupsEnterName: {
		id: 'instance.settings.tabs.general.library-groups.enter-name',
		defaultMessage: 'Enter group name',
	},
	libraryGroupsCreate: {
		id: 'instance.settings.tabs.general.library-groups.create',
		defaultMessage: 'Create new group',
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
	<ConfirmDeleteInstanceModal ref="deleteConfirmModal" @delete="removeProfile" />
	<div class="block">
		<div class="float-end ml-10 relative group w-fit">
			<div class="flex flex-col gap-1">
				<span class="text-lg font-semibold text-contrast">Icon</span>
				<div class="group relative w-fit">
					<OverflowMenu
						v-tooltip="formatMessage(messages.editIcon)"
						class="bg-transparent border-none appearance-none p-0 m-0 cursor-pointer group-active:scale-95 transition-transform"
						:options="[
							{
								id: 'select',
								action: () => setIcon(),
							},
							{
								id: 'remove',
								color: 'danger',
								action: () => resetIcon(),
								shown: !!icon,
							},
						]"
					>
						<Avatar
							:src="icon ? convertFileSrc(icon) : icon"
							size="108px"
							class="transition-[filter] group-hover:brightness-75"
							:tint-by="instance.path"
							no-shadow
						/>
						<div
							class="absolute top-0 h-full w-full flex items-center justify-center opacity-0 transition-all group-hover:opacity-100"
						>
							<EditIcon aria-hidden="true" class="h-10 w-10 text-primary" />
						</div>
						<template #select>
							<UploadIcon />
							{{ icon ? formatMessage(messages.replaceIcon) : formatMessage(messages.selectIcon) }}
						</template>
						<template #remove> <TrashIcon /> {{ formatMessage(messages.removeIcon) }} </template>
					</OverflowMenu>
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
				<ButtonStyled>
					<button
						v-tooltip="installing ? formatMessage(messages.duplicateButtonTooltipInstalling) : null"
						aria-labelledby="duplicate-instance-label"
						:disabled="installing"
						class="w-max !shadow-none"
						@click="duplicateProfile"
					>
						<CopyIcon /> {{ formatMessage(messages.duplicateButton) }}
					</button>
				</ButtonStyled>
				<p class="m-0">
					{{ formatMessage(messages.duplicateInstanceDescription) }}
				</p>
			</div>
		</template>
		<div class="flex flex-col gap-2.5 mt-6">
			<h2 class="m-0 text-lg font-semibold text-contrast block">
				{{ formatMessage(messages.libraryGroups) }}
			</h2>

			<div class="flex flex-col gap-1">
				<Checkbox
					v-for="group in availableGroups"
					:key="group"
					:model-value="groups.includes(group)"
					:label="group"
					@click="toggleGroup(group)"
				/>
				<div class="flex gap-2 items-center">
					<StyledInput
						v-model="newCategoryInput"
						:placeholder="formatMessage(messages.libraryGroupsEnterName)"
						class="w-full max-w-[300px]"
						@submit="() => addCategory"
					/>
					<ButtonStyled>
						<button class="w-fit !shadow-none" @click="() => addCategory()">
							<PlusIcon /> {{ formatMessage(messages.libraryGroupsCreate) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
			<p class="m-0">
				{{ formatMessage(messages.libraryGroupsDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5 mt-6">
			<h2 id="delete-instance-label" class="m-0 text-lg font-semibold text-contrast block">
				{{ formatMessage(messages.deleteInstance) }}
			</h2>
			<ButtonStyled color="red">
				<button
					aria-labelledby="delete-instance-label"
					:disabled="removing"
					class="w-fit !shadow-none"
					@click="deleteConfirmModal.show()"
				>
					<SpinnerIcon v-if="removing" class="animate-spin" />
					<TrashIcon v-else />
					{{
						removing
							? formatMessage(messages.deletingInstanceButton)
							: formatMessage(messages.deleteInstanceButton)
					}}
				</button>
			</ButtonStyled>
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
