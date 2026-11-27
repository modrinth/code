<script setup lang="ts">
import { LibraryIcon, PlusIcon, StarIcon, UserPlusIcon } from '@modrinth/assets'
import { Button, ContextMenu, defineMessages, EmptyState, useVIntl } from '@modrinth/ui'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import Draggable from 'vuedraggable'

import { injectInstanceCreation } from '@/features/instances/instance-creation-context'
import { useInstanceMenu } from '@/features/instances/use-instance-menu'
import type { UninstalledSharedInstance } from '@/features/library/shared-instances/types'
import { useSharedInstanceMenu } from '@/features/library/shared-instances/use-shared-instance-menu'
import type { GameInstance } from '@/helpers/types'
import { FAVORITES_GROUP_ID } from '@/platform/app-lib/instance-groups/types'

import { useInstanceGroupDrag } from '../composables/use-instance-group-drag'
import { useLibraryGroups } from '../composables/use-library-groups'
import { useLibrarySelection } from '../composables/use-library-selection'
import { useLibraryView } from '../composables/use-library-view'
import {
	provideInstanceGroupDrag,
	provideLibraryActions,
	provideLibraryGroups,
	provideLibrarySelection,
	provideLibraryView,
} from '../library-context'
import {
	type InstanceGroup as InstanceGroupType,
	NO_GROUPING_GROUP_ID,
	UNGROUPED_GROUP_ID,
} from '../types'
import GroupInstancesModal from './GroupInstancesModal.vue'
import InstanceGroup from './instance-group/index.vue'
import InstanceGroupDnd from './instance-group-dnd/index.vue'
import LibraryToolbar from './library-toolbar/index.vue'
import LibrarySelectionActionBar from './LibrarySelectionActionBar.vue'

const props = defineProps<{
	instances: GameInstance[]
	sharedInstances: UninstalledSharedInstance[]
}>()

const emit = defineEmits<{
	ready: []
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	library: { id: 'app.library.title', defaultMessage: 'Library' },
	noSearchResults: {
		id: 'app.library.search.no-results.title',
		defaultMessage: 'No instances match your search.',
	},
	instanceActionsLabel: {
		id: 'app.library.instance.actions.label',
		defaultMessage: 'Instance actions',
	},
	emptyHeading: {
		id: 'app.library.empty.heading',
		defaultMessage: 'No instances yet',
	},
	emptyDescription: {
		id: 'app.library.empty.description',
		defaultMessage: 'Create an instance to start playing.',
	},
	emptyAvailableHeading: {
		id: 'app.library.empty.available.heading',
		defaultMessage: 'No shared instances available',
	},
	emptyAvailableDescription: {
		id: 'app.library.empty.available.description',
		defaultMessage:
			"Instances shared with you or by you that you haven't installed yet will appear here.",
	},
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
})

const instanceCreation = injectInstanceCreation()
const { getInstanceOptions } = useInstanceMenu()
const { getSharedInstanceOptions } = useSharedInstanceMenu()

const view = useLibraryView({
	instances: () => props.instances,
	sharedInstances: () => props.sharedInstances,
})
const selection = useLibrarySelection({
	libraryInstances: view.libraryInstances,
	displayedSelections: () =>
		view.visibleInstanceGroups.value.flatMap((group) =>
			group.instances.map((instance) => ({ groupId: group.id, instanceId: instance.id })),
		),
	resetSources: [() => view.displayState.value.group, view.activeTab],
})
const groups = useLibraryGroups(view)
const drag = useInstanceGroupDrag({ view, selection, groups })

provideLibraryView(view)
provideLibrarySelection(selection)
provideLibraryGroups(groups)
provideInstanceGroupDrag(drag)

const instanceOptions = ref<InstanceType<typeof ContextMenu>>()
const groupInstancesModal = ref<InstanceType<typeof GroupInstancesModal>>()

provideLibraryActions({
	openGroupInstancesModal: (groupId) => groupInstancesModal.value?.show(groupId),
	openInstanceContextMenu: (event, item, groupId) => {
		const removeFromGroupId =
			view.displayState.value.group === 'Group' &&
			groupId !== UNGROUPED_GROUP_ID &&
			groupId !== FAVORITES_GROUP_ID
				? groupId
				: null
		instanceOptions.value?.open(event, getInstanceOptions(item, { removeFromGroupId }))
	},
	openSharedInstanceContextMenu: (event, instance) =>
		instanceOptions.value?.open(event, getSharedInstanceOptions(instance)),
})

const { tabInstances, activeTab, groupsLoaded, isSearching, displayState, visibleInstanceGroups } =
	view
const { anchorInstance, toggleWithRange } = selection
const { reorderingGroups, reorderGroups } = groups

const animationsReady = ref(false)
let readyFrame: number | undefined
let unmounted = false
watch(
	groupsLoaded,
	async (loaded) => {
		if (!loaded || animationsReady.value) return
		await nextTick()
		if (unmounted) return
		emit('ready')
		readyFrame = requestAnimationFrame(() => {
			animationsReady.value = true
		})
	},
	{ immediate: true },
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
	(reorderableGroups) => {
		if (!isDraggingGroup.value) {
			const previousGroupTops = getReorderableGroupTops()
			draggableGroups.value = [...reorderableGroups]
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
	unmounted = true
	if (readyFrame !== undefined) cancelAnimationFrame(readyFrame)
	document.documentElement.classList.remove(GROUP_REORDERING_CLASS)
})

function getSelectionAnchorId(groupId: string) {
	return anchorInstance.value?.groupId === groupId ? anchorInstance.value.instanceId : null
}

function handleToggleInstance(groupId: string, instanceId: string, shiftKey: boolean) {
	toggleWithRange({ groupId, instanceId }, shiftKey)
}
</script>

<template>
	<InstanceGroupDnd>
		<section data-library-page-background class="flex flex-col gap-4 pb-16 min-h-[500px]">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.library) }}
			</h2>
			<LibraryToolbar />
			<EmptyState
				v-if="groupsLoaded && activeTab === 'available' && tabInstances.length === 0"
				:icon="UserPlusIcon"
				:heading="formatMessage(messages.emptyAvailableHeading)"
				:description="formatMessage(messages.emptyAvailableDescription)"
			/>
			<EmptyState
				v-else-if="groupsLoaded && tabInstances.length === 0"
				:icon="LibraryIcon"
				:heading="formatMessage(messages.emptyHeading)"
				:description="formatMessage(messages.emptyDescription)"
			>
				<template #actions>
					<Button
						type="colored"
						color="brand"
						size="lg"
						@click="instanceCreation.showCreationModal()"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.newInstance) }}
					</Button>
				</template>
			</EmptyState>
			<div
				v-else-if="groupsLoaded && isSearching && visibleInstanceGroups.length === 0"
				class="text-base text-primary"
			>
				{{ formatMessage(messages.noSearchResults) }}
			</div>
			<Transition
				v-else
				:css="animationsReady"
				enter-active-class="transition-opacity duration-200 ease-out motion-reduce:transition-none"
				enter-from-class="opacity-0"
				enter-to-class="opacity-100"
			>
				<div
					v-if="groupsLoaded && displayState.group === 'Group'"
					ref="libraryGroupsContainer"
					data-library-page-background
					class="flex flex-col"
				>
					<div v-if="visibleFavoritesGroup" class="min-w-0">
						<InstanceGroup
							:animations-ready="animationsReady"
							:instance-group="visibleFavoritesGroup"
							:selection-anchor-instance-id="getSelectionAnchorId(FAVORITES_GROUP_ID)"
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
									:animations-ready="animationsReady"
									:can-drag-reorder="canDragReorderGroups"
									:hide-header="
										instanceGroup.id === UNGROUPED_GROUP_ID && visibleInstanceGroups.length === 1
									"
									:instance-group="instanceGroup"
									:selection-anchor-instance-id="getSelectionAnchorId(instanceGroup.id)"
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
					v-else-if="groupsLoaded"
					:css="animationsReady"
					data-library-page-background
					tag="div"
					class="flex flex-col"
					:move-class="
						animationsReady ? 'transition-transform duration-200 ease-out' : 'transition-none'
					"
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
							:animations-ready="animationsReady"
							:hide-header="
								instanceGroup.id === NO_GROUPING_GROUP_ID && visibleInstanceGroups.length === 1
							"
							:instance-group="instanceGroup"
							:selection-anchor-instance-id="getSelectionAnchorId(instanceGroup.id)"
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
	<GroupInstancesModal ref="groupInstancesModal" />
	<ContextMenu ref="instanceOptions" :label="formatMessage(messages.instanceActionsLabel)">
		<template #remove_from_favorites="{ option }">
			<StarIcon aria-hidden="true" class="fill-current text-primary" />
			{{ option.label }}
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
