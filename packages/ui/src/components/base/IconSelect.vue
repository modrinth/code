<script setup lang="ts">
import { EditIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'

import { Avatar, OverflowMenu } from '../index'

const { formatMessage } = useVIntl()

const icon = defineModel<string | undefined>()
const emit = defineEmits<{
	(e: 'select' | 'reset' | 'remove'): void
}>()

type IconSelectOption = 'select' | 'replace' | 'reset' | 'remove'

withDefaults(
	defineProps<{
		options?: IconSelectOption[]
	}>(),
	{
		options: () => ['select', 'replace', 'reset', 'remove'],
	},
)

const messages = defineMessages({
	editIcon: {
		id: 'icon-select.edit',
		defaultMessage: 'Edit icon',
	},
	selectIcon: {
		id: 'icon-select.select',
		defaultMessage: 'Select icon',
	},
	replaceIcon: {
		id: 'icon-select.replace',
		defaultMessage: 'Replace icon',
	},
	removeIcon: {
		id: 'icon-select.remove',
		defaultMessage: 'Remove icon',
	},
})
</script>

<template>
	<OverflowMenu
		v-tooltip="formatMessage(messages.editIcon)"
		class="m-0 cursor-pointer appearance-none border-none bg-transparent p-0 transition-transform group-active:scale-95"
		:options="[
			{
				id: 'select',
				action: () => emit('select'),
			},
			{
				id: 'remove',
				color: 'danger',
				action: () => emit('remove'),
				shown: !!icon,
			},
		]"
	>
		<Avatar :src="icon" size="108px" class="!border-4 group-hover:brightness-75" no-shadow />
		<div class="absolute right-0 top-0 m-2">
			<div
				class="hovering-icon-shadow m-0 flex aspect-square items-center justify-center rounded-full border-[1px] border-solid border-button-border bg-button-bg p-2 text-primary"
			>
				<EditIcon aria-hidden="true" class="h-4 w-4 text-primary" />
			</div>
		</div>
		<template #select>
			<UploadIcon />
			{{ icon ? formatMessage(messages.replaceIcon) : formatMessage(messages.selectIcon) }}
		</template>
		<template #remove> <TrashIcon /> {{ formatMessage(messages.removeIcon) }} </template>
	</OverflowMenu>
</template>
