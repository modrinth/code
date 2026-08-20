<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EditIcon,
	EyeIcon,
	FolderOpenIcon,
	MinusIcon,
	PaletteIcon,
	PlayIcon,
	PlusIcon,
	StarIcon,
	StopCircleIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, onDeactivated, onUnmounted, ref, toRef, watch } from 'vue'
import Draggable from 'vuedraggable'

import ContextMenu from '@/components/ui/context-menu/index.vue'
import IconEditorModal from '@/components/ui/instance_settings/icon-editor-modal/index.vue'
import GroupInstancesModal from '@/components/ui/library/group-instances-modal.vue'
import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import InstanceGroupDnd from '@/components/ui/library/instance-group/instance-group-dnd.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
import LibrarySelectionActionBar from '@/components/ui/library/LibrarySelectionActionBar.vue'
import {
	getLibraryInstanceSelectionKey,
	type InstanceGroup as InstanceGroupType,
	provideLibrary,
} from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import { FAVORITES_GROUP_ID } from '@/helpers/instance-groups'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	library: { id: 'app.library.title', defaultMessage: 'Library' },
	noSearchResults: {
		id: 'app.library.search.no-results.title',
		defaultMessage: 'No instances match your search.',
	},
	play: { id: 'app.library.instance.action.play', defaultMessage: 'Play' },
	stop: { id: 'app.library.instance.action.stop', defaultMessage: 'Stop' },
	addToFavorites: {
		id: 'app.library.instance.action.add-to-favorites',
		defaultMessage: 'Add to favorites',
	},
	removeFromFavorites: {
		id: 'app.library.instance.action.remove-from-favorites',
		defaultMessage: 'Remove from favorites',
	},
	addContent: { id: 'app.library.instance.action.add-content', defaultMessage: 'Add content' },
	viewInstance: {
		id: 'app.library.instance.action.view-instance',
		defaultMessage: 'View instance',
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
		id: 'app.library.instance.action.duplicate',
		defaultMessage: 'Duplicate instance',
	},
	delete: { id: 'app.library.instance.action.delete', defaultMessage: 'Delete' },
	openFolder: { id: 'app.library.instance.action.open-folder', defaultMessage: 'Open folder' },
	copyPath: { id: 'app.library.instance.action.copy-path', defaultMessage: 'Copy path' },
	removeFromGroup: {
		id: 'app.library.instance.action.remove-from-group',
		defaultMessage: 'Remove from group',
	},
})

const {
	instanceGroups,
	libraryGroupsLoaded,
	isSearching,
	displayState,
	filters,
	reorderingGroups,
	reorderGroups,
	instanceOptions,
	confirmDeleteModal,
	iconEditorModal,
	currentIconEditorInstance,
	currentDeleteInstances,
	clearLibraryInstanceSelection,
	deleteInstance,
	handleInstanceOption,
	handleInstanceIconSaved,
	selectedLibraryInstances,
	setSelectedLibraryInstances,
	toggleLibraryInstanceSelection,
} = provideLibrary(toRef(props, 'instances'))

const hasActiveFilters = computed(() =>
	Object.values(filters.value).some((selectedValues) => selectedValues.length > 0),
)

const visibleInstanceGroups = computed(() =>
	instanceGroups.value.filter((instanceGroup) =>
		instanceGroup.id === FAVORITES_GROUP_ID
			? instanceGroup.instances.length > 0
			: instanceGroup.instances.length > 0 ||
				(!isSearching.value && !hasActiveFilters.value && instanceGroup.key !== 'None'),
	),
)

const visibleReorderableGroups = computed(() =>
	displayState.value.group === 'Group'
		? visibleInstanceGroups.value.filter((group) => group.id !== FAVORITES_GROUP_ID)
		: [],
)
const visibleFavoritesGroup = computed(() =>
	visibleInstanceGroups.value.find((group) => group.id === FAVORITES_GROUP_ID),
)
const draggableGroups = ref<InstanceGroupType[]>([])
const libraryGroupsContainer = ref<HTMLElement>()
const isDraggingGroup = ref(false)
const GROUP_REORDERING_CLASS = 'instance-group-reordering'
const canDragReorderGroups = computed(
	() => !reorderingGroups.value && draggableGroups.value.length > 1,
)

watch(
	visibleReorderableGroups,
	(groups) => {
		if (!isDraggingGroup.value) {
			const previousGroupTops = getReorderableGroupTops()
			draggableGroups.value = [...groups]
			void nextTick(() => animateGroupReorder(previousGroupTops))
		}
	},
	{ immediate: true },
)

function getReorderableGroupTops() {
	const groupTops = new Map<string, number>()
	const groupElements = libraryGroupsContainer.value?.querySelectorAll<HTMLElement>(
		'[data-instance-group-reorder-id]',
	)

	for (const groupElement of groupElements ?? []) {
		const groupId = groupElement.dataset.instanceGroupReorderId
		if (groupId) {
			groupTops.set(groupId, groupElement.getBoundingClientRect().top)
		}
	}

	return groupTops
}

function animateGroupReorder(previousGroupTops: Map<string, number>) {
	if (
		previousGroupTops.size === 0 ||
		window.matchMedia('(prefers-reduced-motion: reduce)').matches
	) {
		return
	}

	const groupElements = libraryGroupsContainer.value?.querySelectorAll<HTMLElement>(
		'[data-instance-group-reorder-id]',
	)

	for (const groupElement of groupElements ?? []) {
		const groupId = groupElement.dataset.instanceGroupReorderId
		const previousTop = groupId ? previousGroupTops.get(groupId) : undefined
		if (previousTop === undefined) continue

		const offset = previousTop - groupElement.getBoundingClientRect().top
		if (Math.abs(offset) < 1) continue

		groupElement.animate(
			[{ transform: `translateY(${offset}px)` }, { transform: 'translateY(0)' }],
			{ duration: 200, easing: 'ease-out' },
		)
	}
}

function onGroupDragStart() {
	isDraggingGroup.value = true
	document.documentElement.classList.add(GROUP_REORDERING_CLASS)
}

function onGroupDragEnd() {
	isDraggingGroup.value = false
	document.documentElement.classList.remove(GROUP_REORDERING_CLASS)

	const currentGroupIds = visibleReorderableGroups.value.map((group) => group.id)
	const orderedGroupIds = draggableGroups.value.map((group) => group.id)
	if (orderedGroupIds.every((groupId, index) => groupId === currentGroupIds[index])) {
		draggableGroups.value = [...visibleReorderableGroups.value]
		return
	}

	void reorderGroups(orderedGroupIds)
}

onUnmounted(() => {
	document.documentElement.classList.remove(GROUP_REORDERING_CLASS)
})

onDeactivated(clearLibraryInstanceSelection)

const anchorInstance = ref<{ groupId: string; instanceId: string } | null>(null)

function handleToggleInstance(groupId: string, instanceId: string, shiftKey: boolean) {
	const displayedInstances = visibleInstanceGroups.value.flatMap((group) =>
		group.instances.map((instance) => ({
			groupId: group.id,
			instanceId: instance.id,
		})),
	)
	const anchor = anchorInstance.value

	if (shiftKey && anchor && displayedInstances.length) {
		const anchorIndex = displayedInstances.findIndex(
			(instance) =>
				instance.groupId === anchor.groupId && instance.instanceId === anchor.instanceId,
		)
		const targetIndex = displayedInstances.findIndex(
			(instance) => instance.groupId === groupId && instance.instanceId === instanceId,
		)

		if (anchorIndex === -1 || targetIndex === -1) {
			toggleLibraryInstanceSelection({ groupId, instanceId })
			return
		}

		const start = Math.min(anchorIndex, targetIndex)
		const end = Math.max(anchorIndex, targetIndex)
		const range = displayedInstances.slice(start, end + 1)
		const nextSelectedInstances = new Map(selectedLibraryInstances.value)
		const targetKey = getLibraryInstanceSelectionKey({ groupId, instanceId })

		if (nextSelectedInstances.has(targetKey)) {
			for (const instance of range) {
				nextSelectedInstances.delete(getLibraryInstanceSelectionKey(instance))
			}
		} else {
			for (const instance of range) {
				nextSelectedInstances.set(getLibraryInstanceSelectionKey(instance), instance)
			}
		}

		setSelectedLibraryInstances(nextSelectedInstances.values())
		anchorInstance.value = null
		return
	}

	toggleLibraryInstanceSelection({ groupId, instanceId })
	anchorInstance.value = { groupId, instanceId }
}

function setInstanceOptions(component: unknown) {
	instanceOptions.value = component as InstanceType<typeof ContextMenu> | null
}

function setConfirmDeleteModal(component: unknown) {
	confirmDeleteModal.value = component as InstanceType<typeof ConfirmDeleteInstanceModal> | null
}

function setIconEditorModal(component: unknown) {
	iconEditorModal.value = component as InstanceType<typeof IconEditorModal> | null
}

watch(selectedLibraryInstances, (selectedInstances) => {
	if (selectedInstances.size === 0) {
		anchorInstance.value = null
		return
	}

	if (
		anchorInstance.value &&
		!selectedInstances.has(getLibraryInstanceSelectionKey(anchorInstance.value))
	) {
		anchorInstance.value = null
	}
})
</script>

<template>
	<InstanceGroupDnd :instances="instances">
		<section data-library-page-background class="flex flex-col gap-3 pb-16 min-h-[500px]">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.library) }}
			</h2>
			<LibraryToolbar />
			<div
				v-if="libraryGroupsLoaded && isSearching && visibleInstanceGroups.length === 0"
				class="text-base text-primary"
			>
				{{ formatMessage(messages.noSearchResults) }}
			</div>
			<Transition
				v-else
				enter-active-class="transition-opacity duration-200 ease-out motion-reduce:transition-none"
				enter-from-class="opacity-0"
				enter-to-class="opacity-100"
			>
				<div
					v-if="libraryGroupsLoaded && displayState.group === 'Group'"
					ref="libraryGroupsContainer"
					data-library-page-background
					class="flex flex-col"
				>
					<div v-if="visibleFavoritesGroup" class="min-w-0">
						<InstanceGroup
							:instance-group="visibleFavoritesGroup"
							:selection-anchor-instance-id="
								anchorInstance?.groupId === FAVORITES_GROUP_ID ? anchorInstance.instanceId : null
							"
							@toggle-selection="
								(instanceId: string, shiftKey: boolean) =>
									handleToggleInstance(FAVORITES_GROUP_ID, instanceId, shiftKey)
							"
						/>
					</div>

					<Draggable
						:list="draggableGroups"
						class="flex flex-col"
						item-key="id"
						:disabled="!canDragReorderGroups"
						:animation="250"
						:swap-threshold="0.75"
						:invert-swap="true"
						:force-fallback="true"
						:fallback-on-body="true"
						:fallback-tolerance="4"
						filter=".instance-group-reorder-ignore, input, textarea, [contenteditable='true']"
						handle=".instance-group-reorder-handle"
						:prevent-on-filter="false"
						ghost-class="instance-group-reorder-ghost"
						chosen-class="instance-group-reorder-chosen"
						drag-class="instance-group-reorder-drag"
						fallback-class="instance-group-reorder-fallback"
						@start="onGroupDragStart"
						@end="onGroupDragEnd"
					>
						<template #item="{ element: instanceGroup }">
							<div
								:key="instanceGroup.id"
								class="min-w-0 w-full"
								:data-instance-group-reorder-id="instanceGroup.id"
							>
								<InstanceGroup
									:can-drag-reorder="canDragReorderGroups"
									:hide-header="visibleInstanceGroups.length === 1"
									:instance-group="instanceGroup"
									:selection-anchor-instance-id="
										anchorInstance?.groupId === instanceGroup.id ? anchorInstance?.instanceId : null
									"
									@toggle-selection="
										(instanceId: string, shiftKey: boolean) =>
											handleToggleInstance(instanceGroup.id, instanceId, shiftKey)
									"
								/>
							</div>
						</template>
					</Draggable>
				</div>

				<TransitionGroup
					v-else-if="libraryGroupsLoaded"
					data-library-page-background
					tag="div"
					class="flex flex-col"
					move-class="transition-transform duration-200 ease-out"
					enter-active-class="transition-[opacity,transform] duration-200 ease-out"
					enter-from-class="opacity-0 -translate-y-2"
					enter-to-class="opacity-100 translate-y-0"
				>
					<div
						v-for="instanceGroup in visibleInstanceGroups"
						:key="instanceGroup.id"
						class="min-w-0"
					>
						<InstanceGroup
							:hide-header="instanceGroup.key === 'None' && visibleInstanceGroups.length === 1"
							:instance-group="instanceGroup"
							:selection-anchor-instance-id="
								anchorInstance?.groupId === instanceGroup.id ? anchorInstance.instanceId : null
							"
							@toggle-selection="
								(instanceId: string, shiftKey: boolean) =>
									handleToggleInstance(instanceGroup.id, instanceId, shiftKey)
							"
						/>
					</div>
				</TransitionGroup>
			</Transition>
		</section>
	</InstanceGroupDnd>
	<LibrarySelectionActionBar />
	<GroupInstancesModal />
	<ConfirmDeleteInstanceModal
		:ref="setConfirmDeleteModal"
		:instances="currentDeleteInstances"
		@delete="deleteInstance"
	/>
	<IconEditorModal
		:ref="setIconEditorModal"
		:instance-id="currentIconEditorInstance?.id"
		:config="currentIconEditorInstance?.icon_config"
		@saved="handleInstanceIconSaved"
	/>
	<ContextMenu :ref="setInstanceOptions" @option-clicked="handleInstanceOption">
		<template #play> <PlayIcon /> {{ formatMessage(messages.play) }} </template>
		<template #stop> <StopCircleIcon /> {{ formatMessage(messages.stop) }} </template>
		<template #add_to_favorites>
			<StarIcon /> {{ formatMessage(messages.addToFavorites) }}
		</template>
		<template #remove_from_favorites>
			<StarIcon style="color: var(--color-text-default); fill: var(--color-text-default)" />
			{{ formatMessage(messages.removeFromFavorites) }}
		</template>
		<template #add_content> <PlusIcon /> {{ formatMessage(messages.addContent) }} </template>
		<template #edit> <EyeIcon /> {{ formatMessage(messages.viewInstance) }} </template>
		<template #edit_icon> <EditIcon /> {{ formatMessage(messages.editIcon) }} </template>
		<template #select_icon> <UploadIcon /> {{ formatMessage(messages.selectIcon) }} </template>
		<template #replace_icon> <UploadIcon /> {{ formatMessage(messages.replaceIcon) }} </template>
		<template #create_icon> <PaletteIcon /> {{ formatMessage(messages.createIcon) }} </template>
		<template #edit_created_icon>
			<PaletteIcon /> {{ formatMessage(messages.editCreatedIcon) }}
		</template>
		<template #remove_icon> <TrashIcon /> {{ formatMessage(messages.removeIcon) }} </template>
		<template #duplicate>
			<ClipboardCopyIcon /> {{ formatMessage(messages.duplicateInstance) }}
		</template>
		<template #delete> <TrashIcon /> {{ formatMessage(messages.delete) }} </template>
		<template #open> <FolderOpenIcon /> {{ formatMessage(messages.openFolder) }} </template>
		<template #copy> <ClipboardCopyIcon /> {{ formatMessage(messages.copyPath) }} </template>
		<template #remove_from_group>
			<MinusIcon /> {{ formatMessage(messages.removeFromGroup) }}
		</template>
	</ContextMenu>
</template>

<style scoped>
:global(.instance-group-reorder-ghost) {
	opacity: 0.35;
}

:global(html.instance-group-reordering),
:global(html.instance-group-reordering *) {
	-webkit-user-select: none !important;
	cursor: grabbing !important;
	user-select: none !important;
}

:global(.instance-group-reorder-fallback) {
	opacity: 0.9;
	pointer-events: none;
}
</style>
