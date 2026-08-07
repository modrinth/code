<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, EditIcon, SquarePlusIcon, TrashIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, useVIntl } from '@modrinth/ui'

defineProps<{
	deleting?: boolean
	canMoveDown: boolean
	canMoveUp: boolean
	onAddToGroup: () => void
	onDeleteGroup: () => void
	onEditGroupName: () => void
	onMoveDown: () => void
	onMoveUp: () => void
}>()

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
		<ButtonStyled circular type="transparent" size="standard">
			<button
				v-tooltip="formatMessage(messages.moveGroupUp)"
				class="!size-8 !min-w-8"
				type="button"
				:aria-label="formatMessage(messages.moveGroupUp)"
				:disabled="!canMoveUp"
				@click.stop="onMoveUp"
			>
				<ArrowUpIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				v-tooltip="formatMessage(messages.moveGroupDown)"
				class="!size-8 !min-w-8"
				type="button"
				:aria-label="formatMessage(messages.moveGroupDown)"
				:disabled="!canMoveDown"
				@click.stop="onMoveDown"
			>
				<ArrowDownIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				v-tooltip="formatMessage(messages.editGroupName)"
				class="!size-8 !min-w-8"
				type="button"
				:aria-label="formatMessage(messages.editGroupName)"
				@click.stop="onEditGroupName"
			>
				<EditIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				v-tooltip="formatMessage(messages.addToGroup)"
				class="!size-8 !min-w-8"
				type="button"
				:aria-label="formatMessage(messages.addToGroup)"
				@click.stop="onAddToGroup"
			>
				<SquarePlusIcon class="!min-w-4 !min-h-4 !size-4" />
			</button>
		</ButtonStyled>
		<ButtonStyled circular type="transparent" size="standard">
			<button
				v-tooltip="formatMessage(messages.deleteGroup)"
				class="!size-8 !min-w-8"
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
