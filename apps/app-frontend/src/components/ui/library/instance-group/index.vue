<script setup lang="ts">
import {
	ClipboardCopyIcon,
	EyeIcon,
	FolderOpenIcon,
	PlayIcon,
	PlusIcon,
	StopCircleIcon,
	TrashIcon,
} from '@modrinth/assets'
import { Accordion } from '@modrinth/ui'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import Instance from '@/components/ui/library/instance-group/instance.vue'
import { useLibrary } from '@/components/ui/library/use-library'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'

const {
	instanceGroups,
	instanceOptions,
	instanceComponents,
	confirmDeleteModal,
	isSectionCollapsed,
	setSectionCollapsed,
	deleteInstance,
	handleInstanceContextMenu,
	handleInstanceOption,
} = useLibrary()
</script>

<template>
	<Accordion
		v-for="instanceGroup in instanceGroups"
		:key="instanceGroup.key"
		:divider="instanceGroup.key !== 'None'"
		:open-by-default="!isSectionCollapsed(instanceGroup.key)"
		class="row"
		@on-open="setSectionCollapsed(instanceGroup.key, false)"
		@on-close="setSectionCollapsed(instanceGroup.key, true)"
	>
		<template v-if="instanceGroup.key !== 'None'" #title>
			<span class="text-base">{{ instanceGroup.key }}</span>
		</template>
		<section class="instances">
			<Instance
				v-for="instance in instanceGroup.instances"
				ref="instanceComponents"
				:key="instance.id + instance.install_stage"
				:instance="instance"
				@contextmenu.prevent.stop="
					(event: MouseEvent) => handleInstanceContextMenu(event, instance.id)
				"
			/>
		</section>
	</Accordion>
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
	</ContextMenu>
</template>

<style lang="scss" scoped>
.row {
	width: 100%;
}

.instances {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
	width: 100%;
	gap: 0.75rem;
	margin-right: auto;
	scroll-behavior: smooth;
	overflow-y: auto;
}
</style>
