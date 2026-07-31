<script setup lang="ts">
import { DropdownIcon, FolderOpenIcon, PlusIcon } from '@modrinth/assets'
import { Button, injectNotificationManager, TeleportOverflowMenu } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useRouter } from 'vue-router'

import { add_project_from_path } from '@/helpers/instance'

const { handleError } = injectNotificationManager()

const props = defineProps({
	instance: {
		type: Object,
		required: true,
	},
})

const router = useRouter()

const handleAddContentFromFile = async () => {
	const newProject = await open({ multiple: true })
	if (!newProject) return

	for (const project of newProject) {
		await add_project_from_path(props.instance.id, project.path ?? project).catch(handleError)
	}
}

const handleSearchContent = async () => {
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
		query: { i: props.instance.id },
	})
}
</script>

<template>
	<div class="joined-buttons">
		<Button @click="handleSearchContent">
			<PlusIcon />
			Install content
		</Button>
		<TeleportOverflowMenu
			label="More options"
			:options="[
				{
					id: 'from_file',
					label: 'Add from file',
					action: handleAddContentFromFile,
				},
			]"
			class="!w-auto !px-2.5 !rounded-xl"
		>
			<DropdownIcon />
			<template #from_file>
				<FolderOpenIcon />
				<span class="no-wrap"> Add from file </span>
			</template>
		</TeleportOverflowMenu>
	</div>
</template>
