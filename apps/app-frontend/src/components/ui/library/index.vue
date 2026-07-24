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
import { toRef } from 'vue'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
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
} = provideLibrary(toRef(props, 'instances'))
</script>

<template>
	<section class="flex flex-col gap-3">
		<h2 class="m-0 text-2xl font-semibold text-contrast">Library</h2>
		<LibraryToolbar />
		<div class="flex flex-col gap-3">
			<InstanceGroup
				v-for="instanceGroup in instanceGroups"
				:key="instanceGroup.id"
				:instance-group="instanceGroup"
			/>
		</div>
	</section>
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
