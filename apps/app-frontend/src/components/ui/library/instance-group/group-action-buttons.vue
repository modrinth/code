<script setup lang="ts">
import { EditIcon, SquarePlusIcon, TrashIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'

defineProps<{
	deleting?: boolean
	onAddToGroup: () => void
	onDeleteGroup: () => void
	onEditGroupName: () => void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	addToGroup: {
		id: 'app.library.group.context-menu.add-to-group',
		defaultMessage: 'Add instances to group',
	},
	editGroupName: {
		id: 'app.library.group.edit-name',
		defaultMessage: 'Edit group name',
	},
	deleteGroup: {
		id: 'app.library.group.delete',
		defaultMessage: 'Delete group',
	},
})
</script>

<template>
	<div
		class="flex shrink-0 items-center opacity-0 transition-opacity duration-250 group-hover/header:opacity-100 focus-within:opacity-100"
	>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				class="!size-8 !min-w-8"
				v-tooltip="formatMessage(messages.editGroupName)"
				type="button"
				:aria-label="formatMessage(messages.editGroupName)"
				@click.stop="onEditGroupName"
			>
				<EditIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				class="!size-8 !min-w-8"
				v-tooltip="formatMessage(messages.addToGroup)"
				type="button"
				:aria-label="formatMessage(messages.addToGroup)"
				@click.stop="onAddToGroup"
			>
				<SquarePlusIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				class="!size-8 !min-w-8"
				v-tooltip="formatMessage(messages.deleteGroup)"
				type="button"
				:aria-label="formatMessage(messages.deleteGroup)"
				:disabled="deleting"
				@click.stop="onDeleteGroup"
			>
				<TrashIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
	</div>
</template>
