<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EyeIcon,
	FolderOpenIcon,
	MinusIcon,
	PlayIcon,
	PlusIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref, toRef, watch } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import InstanceGroupDnd from '@/components/ui/library/instance-group/instance-group-dnd.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
import LibrarySelectionActionBar from '@/components/ui/library/LibrarySelectionActionBar.vue'
import { provideLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

const {
	instanceGroups,
	instanceOptions,
	confirmDeleteModal,
	deleteInstance,
	handleInstanceOption,
	selectedLibraryInstanceIds,
	setSelectedLibraryInstanceIds,
	toggleLibraryInstanceSelection,
} = provideLibrary(toRef(props, 'instances'))

const visibleInstanceGroups = computed(() =>
	instanceGroups.value.filter(
		(instanceGroup) => instanceGroup.key !== 'None' || instanceGroup.instances.length > 0,
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
			toggleLibraryInstanceSelection(instanceId)
			return
		}

		const start = Math.min(anchorIndex, targetIndex)
		const end = Math.max(anchorIndex, targetIndex)
		const range = displayedInstances.slice(start, end + 1)
		const nextSelectedIds = new Set(selectedLibraryInstanceIds.value)

		if (nextSelectedIds.has(instanceId)) {
			for (const instance of range) {
				nextSelectedIds.delete(instance.instanceId)
			}
		} else {
			for (const instance of range) {
				nextSelectedIds.add(instance.instanceId)
			}
		}

		setSelectedLibraryInstanceIds(nextSelectedIds)
		anchorInstance.value = null
		return
	}

	toggleLibraryInstanceSelection(instanceId)
	anchorInstance.value = { groupId, instanceId }
}

watch(
	() => selectedLibraryInstanceIds.value.size,
	(selectedInstanceCount) => {
		if (selectedInstanceCount === 0) {
			anchorInstance.value = null
		}
	},
)
</script>

<template>
	<InstanceGroupDnd :instances="instances">
		<section class="flex flex-col gap-3 pb-16">
			<h2 class="m-0 text-2xl font-semibold text-contrast">Library</h2>
			<LibraryToolbar />
			<div class="flex flex-col">
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
	<ConfirmDeleteInstanceModal ref="confirmDeleteModal" @delete="deleteInstance" />
	<ContextMenu ref="instanceOptions" @option-clicked="handleInstanceOption">
		<template #play> <PlayIcon /> Play </template>
		<template #stop> <StopCircleIcon /> Stop </template>
		<template #add_content> <PlusIcon /> Add content </template>
		<template #edit> <EyeIcon /> View instance </template>
		<template #duplicate> <ClipboardCopyIcon /> Duplicate instance</template>
		<template #delete> <TrashIcon /> Delete </template>
		<template #open> <FolderOpenIcon /> Open folder </template>
		<template #copy> <ClipboardCopyIcon /> Copy path </template>
		<template #remove_from_group> <MinusIcon /> Remove from group </template>
	</ContextMenu>
</template>
