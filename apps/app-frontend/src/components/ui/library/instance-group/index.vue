<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import { DropdownIcon, EditIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	InlineEditableText,
	NewModal,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import Instance from '@/components/ui/library/instance-group/instance.vue'
import type {
	InstanceCard,
	InstanceGroup as InstanceGroupType,
} from '@/components/ui/library/use-library'
import { useLibrary } from '@/components/ui/library/use-library'

const props = withDefaults(
	defineProps<{
		hideHeader?: boolean
		instanceGroup: InstanceGroupType
		selectionAnchorInstanceId?: string | null
	}>(),
	{
		hideHeader: false,
		selectionAnchorInstanceId: null,
	},
)

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const {
	isSectionCollapsed,
	setSectionCollapsed,
	deleteGroup,
	renameGroup,
	handleInstanceContextMenu,
	displayState,
	activeInstanceGroupDrag,
	instanceGroupDragTarget,
	getInstanceGroupDropState,
} = useLibrary()

const instanceComponents = new Map<string, InstanceCard>()
const groupDropTarget = ref<HTMLElement>()
const groupAccordion = ref<InstanceType<typeof Accordion>>()
const groupOptions = ref<InstanceType<typeof ContextMenu>>()
const groupNameInput = ref<InstanceType<typeof InlineEditableText>>()
const confirmDeleteGroupModal = ref<InstanceType<typeof NewModal>>()
const deletingGroup = ref(false)
const groupName = ref(props.instanceGroup.key)
const isUngrouped = computed(() => props.instanceGroup.id === 'group:none')
const groupContextMenuOpen = ref(false)
const isGroupToggleBlocked = computed(
	() => groupContextMenuOpen.value || Boolean(groupNameInput.value?.isEditing),
)
let shouldSkipGroupToggle = false
let groupToggleEventToSkip: MouseEvent | undefined

const emit = defineEmits<{
	(e: 'toggle-selection', instanceId: string, shiftKey: boolean): void
}>()

useDroppable({
	id: computed(() => `instance-group:${props.instanceGroup.id}`),
	element: groupDropTarget,
	disabled: computed(() => displayState.value.group !== 'Group'),
	data: computed(() => ({
		groupId: props.instanceGroup.id,
	})),
})

const messages = defineMessages({
	ungrouped: {
		id: 'app.library.group.ungrouped',
		defaultMessage: 'Ungrouped',
	},
	deleteGroup: {
		id: 'app.library.group.delete',
		defaultMessage: 'Delete group',
	},
	editGroupName: {
		id: 'app.library.group.edit-name',
		defaultMessage: 'Edit group name',
	},
	renameGroupFailed: {
		id: 'app.library.group.rename-failed',
		defaultMessage: 'Unable to rename group',
	},
	groupNameEmpty: {
		id: 'app.library.group.name-empty',
		defaultMessage: 'Group names cannot be empty.',
	},
	groupNameTooLong: {
		id: 'app.library.group.name-too-long',
		defaultMessage: 'Group names cannot be longer than 32 characters.',
	},
	groupNameReserved: {
		id: 'app.library.group.name-reserved',
		defaultMessage: '"None" is reserved and cannot be used as a group name.',
	},
	deleteGroupDescription: {
		id: 'app.library.group.delete-description',
		defaultMessage: 'Instances in this group will be ungrouped.',
	},
})

function openInstanceContextMenu(event: MouseEvent, instanceId: string, instanceGroupId: string) {
	const instanceComponent = instanceComponents.get(instanceId)
	if (!instanceComponent) return

	handleInstanceContextMenu(event, instanceComponent, instanceGroupId)
}

function setInstanceComponent(instanceId: string, component: unknown) {
	if (component) {
		instanceComponents.set(instanceId, component as InstanceCard)
	} else {
		instanceComponents.delete(instanceId)
	}
}

async function removeGroup() {
	if (deletingGroup.value) return

	deletingGroup.value = true
	const deleted = await deleteGroup(props.instanceGroup.id)
	deletingGroup.value = false

	if (deleted) {
		confirmDeleteGroupModal.value?.hide()
	}
}

function requestGroupDeletion() {
	if (isUngrouped.value) return

	if (props.instanceGroup.instances.length > 0) {
		confirmDeleteGroupModal.value?.show()
	} else {
		void removeGroup()
	}
}

function openGroupContextMenu(event: MouseEvent) {
	if (isUngrouped.value) return

	groupContextMenuOpen.value = true
	groupOptions.value?.showMenu(event, props.instanceGroup, [
		{ name: 'edit_name' },
		{ type: 'divider' },
		{ name: 'delete_group', color: 'danger' },
	])
}

function handleGroupOption({ option }: { option: string }) {
	if (option === 'edit_name') {
		void groupNameInput.value?.startEditing()
		return
	}

	if (option === 'delete_group') {
		requestGroupDeletion()
	}
}

function prepareGroupToggle(event: PointerEvent) {
	const editor = groupNameInput.value
	const contextMenuWasOpen = groupContextMenuOpen.value
	if (contextMenuWasOpen) {
		groupOptions.value?.hideMenu()
	}

	shouldSkipGroupToggle = Boolean(
		contextMenuWasOpen ||
		(editor?.isEditing && event.target instanceof Node && !editor.$el.contains(event.target)),
	)
}

function captureGroupClick(event: MouseEvent) {
	groupToggleEventToSkip = shouldSkipGroupToggle ? event : undefined
	shouldSkipGroupToggle = false
}

function toggleGroup(event: MouseEvent) {
	if (event === groupToggleEventToSkip) {
		groupToggleEventToSkip = undefined
		return
	}

	if (groupAccordion.value?.isOpen) {
		groupAccordion.value.close()
	} else {
		groupAccordion.value?.open()
	}
}

function validateGroupName(value: string) {
	const normalizedGroupName = value.trim()
	let reason: string | undefined

	if (normalizedGroupName.length === 0) {
		reason = formatMessage(messages.groupNameEmpty)
	} else if (normalizedGroupName.length > 32) {
		reason = formatMessage(messages.groupNameTooLong)
	} else if (normalizedGroupName.toLowerCase() === 'none') {
		reason = formatMessage(messages.groupNameReserved)
	}

	if (reason) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.renameGroupFailed),
			text: reason,
		})
		return false
	}

	return true
}

async function updateGroupName(value: string) {
	return await renameGroup(props.instanceGroup.id, value)
}

watch(
	() => props.instanceGroup.key,
	(value) => {
		groupName.value = value
	},
)

watch(
	() => props.hideHeader,
	(hideHeader) => {
		if (hideHeader) {
			groupAccordion.value?.open()
		}
	},
	{ flush: 'post' },
)
</script>

<template>
	<div
		ref="groupDropTarget"
		class="instance-group group/instance-container relative select-none pb-3"
		@contextmenu.prevent.stop="openGroupContextMenu"
	>
		<Transition
			enter-active-class="transition-opacity duration-150 ease-out"
			enter-from-class="!opacity-0"
			enter-to-class="opacity-100"
			leave-active-class="transition-opacity duration-150 ease-in"
			leave-from-class="opacity-100"
			leave-to-class="!opacity-0"
		>
			<div
				v-if="
					activeInstanceGroupDrag &&
					instanceGroupDragTarget === instanceGroup.id &&
					getInstanceGroupDropState(instanceGroup.id).canDrop
				"
				class="pointer-events-none absolute -inset-2 inset-y-0 z-20 rounded-xl border-2 opacity-50 border-dashed border-brand bg-transparent"
			/>
		</Transition>
		<div
			v-if="!hideHeader"
			class="group/header h-10 flex w-full items-center gap-2 border-0 border-b border-solid border-b-surface-5"
		>
			<div
				class="group/open-target flex min-w-0 items-center gap-2"
				:class="isGroupToggleBlocked ? 'cursor-default' : 'cursor-pointer'"
				@click="toggleGroup"
				@click.capture="captureGroupClick"
				@pointerdown.capture="prepareGroupToggle"
			>
				<button
					class="flex shrink-0 items-center border-0 bg-transparent p-0"
					:class="isGroupToggleBlocked ? 'cursor-default' : 'cursor-pointer'"
					type="button"
					:aria-expanded="groupAccordion?.isOpen"
					:aria-label="groupAccordion?.isOpen ? 'Collapse group' : 'Expand group'"
					@click.stop="toggleGroup"
				>
					<DropdownIcon
						class="size-5 shrink-0 text-secondary transition-all duration-300 group-hover/open-target:text-primary"
						:class="{ 'rotate-180': groupAccordion?.isOpen }"
					/>
				</button>
				<InlineEditableText
					v-if="!isUngrouped"
					ref="groupNameInput"
					v-model="groupName"
					activation-mode="icon"
					class="text-base font-semibold !h-10 text-primary select-none group-hover/open-target:text-contrast"
					:edit-label="formatMessage(commonMessages.renameButton)"
					max-width="24rem"
					icon-text-class="select-none"
					:max-length="32"
					:on-change="updateGroupName"
					:validate="validateGroupName"
				/>
				<span
					v-else
					class="text-base font-semibold text-primary select-none group-hover/open-target:text-contrast"
				>
					{{ formatMessage(messages.ungrouped) }}
				</span>
				<TagItem
					v-if="instanceGroup.instances.length"
					class="shrink-0 border-surface-3 bg-surface-2"
				>
					{{ instanceGroup.instances.length }}
				</TagItem>
			</div>
			<div class="min-w-0 flex-1" />
			<ButtonStyled v-if="!isUngrouped" circular type="transparent">
				<button
					v-tooltip="formatMessage(messages.deleteGroup)"
					class="opacity-0 !transition-all duration-150 group-hover/instance-container:opacity-100 -m-1.5"
					type="button"
					:aria-label="formatMessage(messages.deleteGroup)"
					:disabled="deletingGroup"
					@click.stop="requestGroupDeletion"
				>
					<TrashIcon class="!size-4 !min-h-4 !min-w-4" />
				</button>
			</ButtonStyled>
		</div>
		<Accordion
			ref="groupAccordion"
			:open-by-default="hideHeader || !isSectionCollapsed(instanceGroup.id)"
			class="w-full"
			@on-open="setSectionCollapsed(instanceGroup.id, false)"
			@on-close="setSectionCollapsed(instanceGroup.id, true)"
		>
			<section
				class="grid min-h-[45px] mt-2.5 w-full grid-cols-[repeat(auto-fill,minmax(20rem,22rem))] gap-3 overflow-y-auto scroll-smooth"
			>
				<div v-for="instance in instanceGroup.instances" :key="instance.id" class="min-w-0 w-full">
					<Instance
						:ref="(component: unknown) => setInstanceComponent(instance.id, component)"
						:instance="instance"
						:instance-group-id="instanceGroup.id"
						:is-selection-anchor="selectionAnchorInstanceId === instance.id"
						@toggle-selection="
							(shiftKey: boolean) => emit('toggle-selection', instance.id, shiftKey)
						"
						@contextmenu.prevent.stop="
							(event: MouseEvent) => openInstanceContextMenu(event, instance.id, instanceGroup.id)
						"
					/>
				</div>
				<p
					v-if="instanceGroup.instances.length === 0"
					class="col-span-full m-0 py-2.5 pl-0.5 text-base font-medium text-secondary"
				>
					No instances in this group.
				</p>
			</section>
		</Accordion>
	</div>

	<ContextMenu
		ref="groupOptions"
		@menu-closed="groupContextMenuOpen = false"
		@option-clicked="handleGroupOption"
	>
		<template #edit_name> <EditIcon /> {{ formatMessage(messages.editGroupName) }} </template>
		<template #delete_group> <TrashIcon /> {{ formatMessage(messages.deleteGroup) }} </template>
	</ContextMenu>

	<NewModal
		ref="confirmDeleteGroupModal"
		:header="formatMessage(messages.deleteGroup)"
		fade="danger"
		width="500px"
	>
		<p class="m-0 text-base text-primary">
			{{ formatMessage(messages.deleteGroupDescription) }}
		</p>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button type="button" @click="confirmDeleteGroupModal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="red">
					<button type="button" :disabled="deletingGroup" @click="removeGroup">
						<TrashIcon />
						{{ formatMessage(messages.deleteGroup) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
