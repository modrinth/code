<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, EditIcon, SquarePlusIcon, TrashIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'

withDefaults(
	defineProps<{
		deleting?: boolean
		canMoveDown: boolean
		canMoveUp: boolean
		onAddToGroup: () => void
		onDeleteGroup: () => void
		onEditGroupName: () => void
		onMoveDown: () => void
		onMoveUp: () => void
		showDelete?: boolean
		showEdit?: boolean
	}>(),
	{
		deleting: false,
		showDelete: true,
		showEdit: true,
	},
)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	addToGroup: {
		id: 'app.library.group.context-menu.add-to-group',
		defaultMessage: 'Add to group',
	},
	editGroupName: {
		id: 'app.library.group.edit-name',
		defaultMessage: 'Edit group name',
	},
	deleteGroup: {
		id: 'app.library.group.delete',
		defaultMessage: 'Delete group',
	},
	moveGroupUp: {
		id: 'app.library.group.move-up',
		defaultMessage: 'Move group up',
	},
	moveGroupDown: {
		id: 'app.library.group.move-down',
		defaultMessage: 'Move group down',
	},
})
</script>

<template>
	<div
		class="instance-group-reorder-ignore flex shrink-0 items-center opacity-0 transition-opacity duration-250 group-hover/header:opacity-100 focus-within:opacity-100"
	>
		<IconButton
			v-tooltip="formatMessage(messages.moveGroupUp)"
			:label="formatMessage(messages.moveGroupUp)"
			type="quiet"
			size="sm"
			:disabled="!canMoveUp"
			@click.stop="onMoveUp"
		>
			<ArrowUpIcon />
		</IconButton>
		<IconButton
			v-tooltip="formatMessage(messages.moveGroupDown)"
			:label="formatMessage(messages.moveGroupDown)"
			type="quiet"
			size="sm"
			:disabled="!canMoveDown"
			@click.stop="onMoveDown"
		>
			<ArrowDownIcon />
		</IconButton>
		<IconButton
			v-if="showEdit"
			v-tooltip="formatMessage(messages.editGroupName)"
			:label="formatMessage(messages.editGroupName)"
			type="quiet"
			size="sm"
			@click.stop="onEditGroupName"
		>
			<EditIcon />
		</IconButton>
		<IconButton
			v-tooltip="formatMessage(messages.addToGroup)"
			:label="formatMessage(messages.addToGroup)"
			type="quiet"
			size="sm"
			@click.stop="onAddToGroup"
		>
			<SquarePlusIcon />
		</IconButton>
		<IconButton
			v-if="showDelete"
			v-tooltip="formatMessage(messages.deleteGroup)"
			:label="formatMessage(messages.deleteGroup)"
			type="quiet"
			size="sm"
			:disabled="deleting"
			@click.stop="onDeleteGroup"
		>
			<TrashIcon />
		</IconButton>
	</div>
</template>
