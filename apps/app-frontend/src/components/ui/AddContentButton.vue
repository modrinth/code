<script setup lang="ts">
import { DropdownIcon, FolderOpenIcon, PlusIcon } from '@modrinth/assets'
import Button from '@modrinth/ui/src/components/base/buttons/Button.vue'
import {
	ButtonGroup,
	defineMessages,
	injectNotificationManager,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useRouter } from 'vue-router'

import { add_project_from_path } from '@/helpers/instance'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	installContent: { id: 'app.instance.content.install', defaultMessage: 'Install content' },
	addFromFile: { id: 'app.instance.content.add-from-file', defaultMessage: 'Add from file' },
	moreInstallOptions: {
		id: 'app.instance.content.more-install-options',
		defaultMessage: 'More install options',
	},
})

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
	<ButtonGroup>
		<Button @click="handleSearchContent">
			<PlusIcon aria-hidden="true" />
			{{ formatMessage(messages.installContent) }}
		</Button>
		<TeleportOverflowMenu
			:label="formatMessage(messages.moreInstallOptions)"
			:options="[
				{
					id: 'from_file',
					label: formatMessage(messages.addFromFile),
					icon: FolderOpenIcon,
					action: handleAddContentFromFile,
				},
			]"
		>
			<DropdownIcon aria-hidden="true" />
		</TeleportOverflowMenu>
	</ButtonGroup>
</template>
