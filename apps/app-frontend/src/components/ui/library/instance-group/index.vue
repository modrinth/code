<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import {
	DropdownIcon,
	EditIcon,
	PlusIcon,
	SquarePlusIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	InlineEditableText,
	NewModal,
	TagItem,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, nextTick, onActivated, onDeactivated, onMounted, ref, watch } from 'vue'

import ContextMenu from '@/components/ui/context-menu/index.vue'
import GroupActionButtons from '@/components/ui/library/instance-group/group-action-buttons.vue'
import InstanceCard from '@/components/ui/library/instance-group/instance-card.vue'
import type {
	InstanceCard as InstanceCardExposed,
	InstanceGroup as InstanceGroupType,
} from '@/components/ui/library/use-library'
import { useLibrary } from '@/components/ui/library/use-library'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { FAVORITES_GROUP_ID, MAX_INSTANCE_GROUP_NAME_LENGTH } from '@/helpers/instance-groups'

const INSTANCE_GRID_OBSERVER_ACTIVATION_DELAY = 500

const props = withDefaults(
	defineProps<{
		canDragReorder?: boolean
		hideHeader?: boolean
		instanceGroup: InstanceGroupType
		selectionAnchorInstanceId?: string | null
	}>(),
	{
		canDragReorder: false,
		hideHeader: false,
		selectionAnchorInstanceId: null,
	},
)

const { formatMessage } = useVIntl()
const appSettings = useAppSettings()
const compactMode = computed(() => appSettings.getFeatureFlag('compact_instance_cards'))
const { addNotification } = injectNotificationManager()
const {
	isSectionCollapsed,
	setSectionCollapsed,
	isSearching,
	deleteGroup,
	renameGroup,
	canMoveGroupUp,
	canMoveGroupDown,
	moveGroup,
	groupIdPendingNameEdit,
	completePendingGroupNameEdit,
	handleInstanceContextMenu,
	openGroupInstancesModal,
	displayState,
	activeInstanceGroupDrag,
	instanceGroupDragTarget,
	getInstanceGroupDropState,
} = useLibrary()
const showCreationModal = inject<() => void>('showCreationModal')

const instanceComponents = new Map<string, InstanceCardExposed>()
const groupDropTarget = ref<HTMLElement>()
const groupAccordion = ref<InstanceType<typeof Accordion>>()
const groupOptions = ref<InstanceType<typeof ContextMenu>>()
const groupNameInput = ref<InstanceType<typeof InlineEditableText>>()
const confirmDeleteGroupModal = ref<InstanceType<typeof NewModal>>()
const instanceGridContent = ref<HTMLElement>()
const instanceGridHeight = ref<number>()
const deletingGroup = ref(false)
const groupName = ref(props.instanceGroup.key)
const isUngrouped = computed(() => props.instanceGroup.id === 'group:none')
const isFavorites = computed(() => props.instanceGroup.id === FAVORITES_GROUP_ID)
const isCustomGroup = computed(
	() => displayState.value.group === 'Group' && !isUngrouped.value && !isFavorites.value,
)
const isReorderableGroup = computed(
	() => displayState.value.group === 'Group' && !isFavorites.value,
)
const groupContextMenuOpen = ref(false)
const isGroupToggleBlocked = computed(
	() => isSearching.value || groupContextMenuOpen.value || Boolean(groupNameInput.value?.isEditing),
)
let shouldSkipGroupToggle = false
let groupToggleEventToSkip: MouseEvent | undefined
let instanceGridResizeObserver: ResizeObserver | undefined
let instanceGridObserverActivationTimeout: ReturnType<typeof setTimeout> | undefined

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
	favorites: {
		id: 'app.library.group.favorites',
		defaultMessage: 'Favorites',
	},
	collapseGroup: { id: 'app.library.group.collapse', defaultMessage: 'Collapse group' },
	expandGroup: { id: 'app.library.group.expand', defaultMessage: 'Expand group' },
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
		defaultMessage: 'Group names cannot be longer than {maxLength} characters.',
	},
	groupNameReserved: {
		id: 'app.library.group.name-reserved',
		defaultMessage: '"None" is reserved and cannot be used as a group name.',
	},
	deleteGroupDescription: {
		id: 'app.library.group.delete-description',
		defaultMessage: 'Instances in this group will be ungrouped.',
	},
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
	addToGroup: {
		id: 'app.library.group.context-menu.add-to-group',
		defaultMessage: 'Add to group',
	},
	emptyGroup: {
		id: 'app.library.group.empty',
		defaultMessage: 'Drag and drop to add instances.',
	},
})

function openInstanceContextMenu(event: MouseEvent, instanceId: string, instanceGroupId: string) {
	const instanceComponent = instanceComponents.get(instanceId)
	if (!instanceComponent) return

	handleInstanceContextMenu(event, instanceComponent, instanceGroupId)
}

function setInstanceComponent(instanceId: string, component: unknown) {
	if (component) {
		instanceComponents.set(instanceId, component as InstanceCardExposed)
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
	if (isUngrouped.value || isFavorites.value) return

	if (props.instanceGroup.instances.length > 0) {
		confirmDeleteGroupModal.value?.show()
	} else {
		void removeGroup()
	}
}

function openGroupContextMenu(event: MouseEvent) {
	groupContextMenuOpen.value = true
	groupOptions.value?.showMenu(
		event,
		props.instanceGroup,
		isFavorites.value
			? [{ name: 'new_instance' }]
			: isCustomGroup.value
				? [
						{ name: 'new_instance' },
						{ name: 'add_instances' },
						{ name: 'edit_name' },
						{ type: 'divider' },
						{ name: 'delete_group', color: 'danger' },
					]
				: [{ name: 'new_instance' }],
	)
}

function handleGroupOption({ option }: { option: string }) {
	if (option === 'new_instance') {
		showCreationModal?.()
		return
	}

	if (option === 'add_instances') {
		openGroupInstancesModal(props.instanceGroup.id)
		return
	}

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
	if (isSearching.value) return

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
	} else if (normalizedGroupName.length > MAX_INSTANCE_GROUP_NAME_LENGTH) {
		reason = formatMessage(messages.groupNameTooLong, {
			maxLength: MAX_INSTANCE_GROUP_NAME_LENGTH,
		})
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
	[() => props.instanceGroup.id, groupIdPendingNameEdit],
	async ([groupId, pendingGroupId]) => {
		if (groupId !== pendingGroupId) return

		await nextTick()
		groupDropTarget.value?.scrollIntoView({ block: 'nearest' })
		await groupNameInput.value?.startEditing()
		completePendingGroupNameEdit(groupId)
	},
	{ immediate: true, flush: 'post' },
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

function stopInstanceGridResizeObserver() {
	clearTimeout(instanceGridObserverActivationTimeout)
	instanceGridResizeObserver?.disconnect()
}

function startInstanceGridResizeObserver() {
	stopInstanceGridResizeObserver()
	instanceGridHeight.value = undefined
	instanceGridObserverActivationTimeout = setTimeout(() => {
		instanceGridObserverActivationTimeout = undefined

		const gridContent = instanceGridContent.value
		if (!gridContent?.isConnected) return

		instanceGridHeight.value = gridContent.getBoundingClientRect().height
		instanceGridResizeObserver = new ResizeObserver(() => {
			if (!gridContent.isConnected) return
			instanceGridHeight.value = gridContent.getBoundingClientRect().height
		})
		instanceGridResizeObserver.observe(gridContent)
	}, INSTANCE_GRID_OBSERVER_ACTIVATION_DELAY)
}

onActivated(startInstanceGridResizeObserver)
onDeactivated(stopInstanceGridResizeObserver)
onMounted(startInstanceGridResizeObserver)
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
				class="pointer-events-none absolute -inset-2 inset-y-0 z-20 rounded-xl border-2 opacity-40 border-dashed border-contrast bg-transparent"
			/>
		</Transition>
		<div
			v-if="!hideHeader"
			class="group/header h-10 flex w-full items-center gap-2 border-0 border-b border-solid border-b-surface-5"
			:class="{
				'instance-group-reorder-handle': isReorderableGroup && canDragReorder,
			}"
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
					:aria-label="
						formatMessage(groupAccordion?.isOpen ? messages.collapseGroup : messages.expandGroup)
					"
					@click.stop="toggleGroup"
				>
					<DropdownIcon
						class="size-5 shrink-0 text-secondary transition-all duration-300 group-hover/open-target:text-primary"
						:class="{ 'rotate-180': groupAccordion?.isOpen }"
					/>
				</button>
				<InlineEditableText
					v-if="!isUngrouped && !isFavorites"
					ref="groupNameInput"
					v-model="groupName"
					activation-mode="manual"
					class="text-base font-semibold !h-10 text-primary select-none group-hover/open-target:text-contrast"
					:edit-label="formatMessage(commonMessages.renameButton)"
					max-width="24rem"
					icon-text-class="select-none"
					:max-length="MAX_INSTANCE_GROUP_NAME_LENGTH"
					:on-change="updateGroupName"
					:validate="validateGroupName"
				/>
				<span
					v-else-if="isUngrouped"
					class="text-base font-semibold text-primary select-none group-hover/open-target:text-contrast"
				>
					{{ formatMessage(messages.ungrouped) }}
				</span>
				<span
					v-else
					class="flex items-center gap-1.5 text-base font-semibold text-primary select-none group-hover/open-target:text-contrast"
				>
					{{ formatMessage(messages.favorites) }}
				</span>
				<TagItem
					v-if="instanceGroup.instances.length"
					class="shrink-0 border-surface-3 bg-surface-2"
				>
					{{ instanceGroup.instances.length }}
				</TagItem>
			</div>
			<div class="min-w-0 flex-1" />
			<GroupActionButtons
				v-if="isCustomGroup || isUngrouped"
				:can-move-down="canMoveGroupDown(instanceGroup.id)"
				:can-move-up="canMoveGroupUp(instanceGroup.id)"
				:deleting="deletingGroup"
				:on-add-to-group="() => openGroupInstancesModal(instanceGroup.id)"
				:on-delete-group="requestGroupDeletion"
				:on-edit-group-name="() => groupNameInput?.startEditing()"
				:on-move-down="() => moveGroup(instanceGroup.id, 1)"
				:on-move-up="() => moveGroup(instanceGroup.id, -1)"
				:show-delete="!isUngrouped"
				:show-edit="!isUngrouped"
			/>
		</div>
		<Accordion
			ref="groupAccordion"
			:open-by-default="hideHeader || !isSectionCollapsed(instanceGroup.id)"
			:force-open="isSearching"
			class="w-full"
			@on-open="setSectionCollapsed(instanceGroup.id, false)"
			@on-close="setSectionCollapsed(instanceGroup.id, true)"
		>
			<div
				class="mt-2.5 overflow-hidden transition-[height] duration-200 ease-out motion-reduce:transition-none"
				:style="{
					height: instanceGridHeight === undefined ? undefined : `${instanceGridHeight}px`,
				}"
			>
				<div ref="instanceGridContent">
					<TransitionGroup
						tag="section"
						class="grid min-h-[45px] w-full gap-3 overflow-y-auto scroll-smooth"
						:class="
							compactMode
								? 'grid-cols-[repeat(auto-fill,minmax(min(15rem,100%),1fr))]'
								: 'grid-cols-[repeat(auto-fill,minmax(min(10rem,100%),1fr))] max-xl:grid-cols-[repeat(auto-fill,minmax(min(8rem,100%),1fr))]'
						"
						move-class="transition-transform duration-200 ease-out motion-reduce:transition-none"
						enter-active-class="transition-[opacity,transform] duration-[150ms] ease-out motion-reduce:transition-none"
						enter-from-class="opacity-0"
						enter-to-class="opacity-100 scale-100"
					>
						<div
							v-for="instance in instanceGroup.instances"
							:key="instance.id"
							class="min-w-0 w-full"
						>
							<InstanceCard
								:ref="(component: unknown) => setInstanceComponent(instance.id, component)"
								:instance="instance"
								:instance-group-id="instanceGroup.id"
								:is-selection-anchor="selectionAnchorInstanceId === instance.id"
								@toggle-selection="
									(shiftKey: boolean) => emit('toggle-selection', instance.id, shiftKey)
								"
								@contextmenu.prevent.stop="
									(event: MouseEvent) =>
										openInstanceContextMenu(event, instance.id, instanceGroup.id)
								"
							/>
						</div>
						<p
							v-if="instanceGroup.instances.length === 0"
							key="empty-group"
							class="col-span-full m-0 pt-1 pl-0.5 text-base font-base text-secondary opacity-80"
						>
							{{ formatMessage(messages.emptyGroup) }}
						</p>
					</TransitionGroup>
				</div>
			</div>
		</Accordion>
	</div>

	<ContextMenu
		ref="groupOptions"
		@menu-closed="groupContextMenuOpen = false"
		@option-clicked="handleGroupOption"
	>
		<template #new_instance> <PlusIcon /> {{ formatMessage(messages.newInstance) }} </template>
		<template #add_instances>
			<SquarePlusIcon /> {{ formatMessage(messages.addToGroup) }}
		</template>
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
				<Button type="outlined" @click="confirmDeleteGroupModal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button type="colored" color="red" :disabled="deletingGroup" @click="removeGroup">
					<TrashIcon />
					{{ formatMessage(messages.deleteGroup) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
