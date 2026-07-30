<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EyeIcon,
	FolderOpenIcon,
	MinusIcon,
	PlayIcon,
	PlusIcon,
	StarIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref, toRef, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import GroupInstancesModal from '@/components/ui/library/group-instances-modal.vue'
import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import InstanceGroupDnd from '@/components/ui/library/instance-group/instance-group-dnd.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
import LibrarySelectionActionBar from '@/components/ui/library/LibrarySelectionActionBar.vue'
import { getLibraryInstanceSelectionKey, provideLibrary } from '@/components/ui/library/use-library'
import { FAVORITES_GROUP_ID } from '@/helpers/instance-groups'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

const {
	instanceGroups,
	filters,
	instanceOptions,
	confirmDeleteModal,
	deleteInstance,
	handleInstanceOption,
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
				(!hasActiveFilters.value && instanceGroup.key !== 'None'),
	),
)

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
		<section data-library-page-background class="flex flex-col gap-3 pb-16">
			<h2 class="m-0 text-2xl font-semibold text-contrast">Library</h2>
			<LibraryToolbar />
			<div data-library-page-background class="flex flex-col">
				<InstanceGroup
					v-for="instanceGroup in visibleInstanceGroups"
					:key="instanceGroup.id"
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
		</section>
	</InstanceGroupDnd>
	<LibrarySelectionActionBar />
	<GroupInstancesModal />
	<ConfirmDeleteInstanceModal :ref="setConfirmDeleteModal" @delete="deleteInstance" />
	<ContextMenu :ref="setInstanceOptions" @option-clicked="handleInstanceOption">
		<template #play> <PlayIcon /> Play </template>
		<template #stop> <StopCircleIcon /> Stop </template>
		<template #add_to_favorites> <StarIcon /> Add to favorites </template>
		<template #remove_from_favorites>
			<StarIcon style="color: var(--color-text-default); fill: var(--color-text-default)" /> Remove
			from favorites
		</template>
		<template #add_content> <PlusIcon /> Add content </template>
		<template #edit> <EyeIcon /> View instance </template>
		<template #duplicate> <ClipboardCopyIcon /> Duplicate instance</template>
		<template #delete> <TrashIcon /> Delete </template>
		<template #open> <FolderOpenIcon /> Open folder </template>
		<template #copy> <ClipboardCopyIcon /> Copy path </template>
		<template #remove_from_group> <MinusIcon /> Remove from group </template>
	</ContextMenu>
</template>
