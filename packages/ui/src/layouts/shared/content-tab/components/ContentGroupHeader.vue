<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, DropdownIcon, EditIcon, TrashIcon } from '@modrinth/assets'
import { nextTick, ref, watch } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import InlineEditableText from '#ui/components/base/InlineEditableText.vue'
import TagItem from '#ui/components/base/TagItem.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

const props = defineProps<{
	name: string
	count: number
	collapsed: boolean
	editable: boolean
	startEditing: boolean
	maxNameLength?: number
	canMoveUp: boolean
	canMoveDown: boolean
	disabled?: boolean
	onRename: (name: string) => Promise<boolean>
}>()

const emit = defineEmits<{
	toggle: []
	moveUp: []
	moveDown: []
	delete: [event: MouseEvent]
	editingStarted: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	collapse: { id: 'content.groups.collapse', defaultMessage: 'Collapse group' },
	expand: { id: 'content.groups.expand', defaultMessage: 'Expand group' },
	rename: { id: 'content.groups.rename', defaultMessage: 'Rename group' },
	delete: { id: 'content.groups.delete', defaultMessage: 'Delete group' },
	moveUp: { id: 'content.groups.move-up', defaultMessage: 'Move group up' },
	moveDown: { id: 'content.groups.move-down', defaultMessage: 'Move group down' },
})

const nameInput = ref<InstanceType<typeof InlineEditableText>>()
const nameModel = ref(props.name)

watch(
	() => props.name,
	(name) => {
		nameModel.value = name
	},
)

watch(
	() => props.startEditing,
	async (start) => {
		if (!start || !props.editable) return
		await nextTick()
		await nameInput.value?.startEditing()
		emit('editingStarted')
	},
	{ immediate: true, flush: 'post' },
)

function startNameEditing() {
	if (!nameInput.value?.isEditing) void nameInput.value?.startEditing()
}

function validateName(value: string) {
	return value.trim().length > 0
}

async function rename(value: string) {
	const succeeded = await props.onRename(value)
	if (!succeeded) nameModel.value = props.name
	return succeeded
}
</script>

<template>
	<div class="group/content-group flex h-full w-full min-w-0 items-center gap-1.5">
		<button
			type="button"
			class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0 text-secondary transition-colors hover:text-primary"
			:aria-expanded="!collapsed"
			:aria-label="formatMessage(collapsed ? messages.expand : messages.collapse)"
			@click="emit('toggle')"
		>
			<DropdownIcon
				aria-hidden="true"
				class="size-5 transition-transform duration-200"
				:class="{ 'rotate-180': !collapsed }"
			/>
		</button>
		<div
			class="flex min-w-0 items-center gap-1.5 font-semibold text-primary transition-colors group-hover/content-group:text-contrast"
		>
			<InlineEditableText
				v-if="editable"
				ref="nameInput"
				v-model="nameModel"
				activation-mode="manual"
				class="!h-6 cursor-text select-none text-base font-semibold"
				icon-text-class="select-none"
				max-width="24rem"
				:edit-label="formatMessage(commonMessages.renameButton)"
				:max-length="maxNameLength"
				:validate="validateName"
				:on-change="rename"
				@click="startNameEditing"
			/>
			<span v-else class="truncate">{{ name }}</span>
			<TagItem v-if="count" class="shrink-0 border-surface-3 bg-surface-2 tabular-nums">
				{{ count }}
			</TagItem>
		</div>
		<button
			type="button"
			tabindex="-1"
			aria-hidden="true"
			class="h-full min-w-0 flex-1 cursor-pointer border-0 bg-transparent p-0"
			@click="emit('toggle')"
		/>
		<div
			v-if="editable"
			class="flex shrink-0 items-center opacity-0 transition-opacity duration-150 focus-within:opacity-100 group-hover/content-group:opacity-100"
		>
			<IconButton
				v-tooltip="formatMessage(messages.moveUp)"
				:label="formatMessage(messages.moveUp)"
				type="quiet"
				size="sm"
				:disabled="disabled || !canMoveUp"
				@click="emit('moveUp')"
			>
				<ArrowUpIcon aria-hidden="true" />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.moveDown)"
				:label="formatMessage(messages.moveDown)"
				type="quiet"
				size="sm"
				:disabled="disabled || !canMoveDown"
				@click="emit('moveDown')"
			>
				<ArrowDownIcon aria-hidden="true" />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.rename)"
				:label="formatMessage(messages.rename)"
				type="quiet"
				size="sm"
				:disabled="disabled"
				@click="nameInput?.startEditing()"
			>
				<EditIcon aria-hidden="true" />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.delete)"
				:label="formatMessage(messages.delete)"
				type="quiet"
				size="sm"
				:disabled="disabled"
				@click="(event: MouseEvent) => emit('delete', event)"
			>
				<TrashIcon aria-hidden="true" />
			</IconButton>
		</div>
	</div>
</template>
