<script setup lang="ts">
import {
	ArrowLeftRightIcon,
	DownloadIcon,
	Link2Icon,
	LockIcon,
	MoreVerticalIcon,
	TrashExclamationIcon,
	TrashIcon,
} from '@modrinth/assets'
import { useMagicKeys } from '@vueuse/core'
import { computed, ref } from 'vue'

import { IconButton, TeleportOverflowMenu } from '#ui/components/base/buttons'
import Toggle from '#ui/components/base/Toggle.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentCardTableItem } from '../../types'
import type { ContentTableAction } from './types'
import { showsRemoveAction, showsToggleAction, showsVersionAction } from './use-table-layout'

const props = defineProps<{
	row: ContentCardTableItem
	actions: Set<ContentTableAction>
	title: string
	disabled: boolean
}>()

const emit = defineEmits<{
	toggle: [value: boolean]
	remove: [event: MouseEvent]
	update: []
	switchVersion: []
	prefetch: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	frozen: {
		id: 'content.card.frozen',
		defaultMessage: 'This project is locked to its current version until unfrozen.',
	},
	synced: {
		id: 'content.card.synced',
		defaultMessage: 'Synced across instances',
	},
	syncUpdatePending: {
		id: 'content.card.sync-update-pending',
		defaultMessage:
			'Some synced copies are waiting for changes. An instance may be running, have a frozen or incompatible version, or already contain its own copy.',
	},
})

const { shift: shiftHeld } = useMagicKeys()
const deleteHovered = ref(false)

const showVersionSlot = computed(() => showsVersionAction(props.row, props.actions))
const showToggle = computed(() => showsToggleAction(props.row, props.actions))
const showRemove = computed(() => showsRemoveAction(props.row, props.actions))
const toggleDisabled = computed(() => props.disabled || !!props.row.toggleDisabled)
const toggleTooltip = computed(() => {
	if (toggleDisabled.value) {
		return props.row.toggleDisabledTooltip ?? props.row.disabledTooltip ?? undefined
	}
	return formatMessage(
		props.row.enabled === false ? commonMessages.enableButton : commonMessages.disableButton,
	)
})
const syncStatusLabel = computed(() =>
	formatMessage(props.row.syncUpdatePending ? messages.syncUpdatePending : messages.synced),
)

function actionLabel(label: string) {
	return props.disabled && props.row.disabledTooltip ? props.row.disabledTooltip : label
}

const deleteLabel = computed(() =>
	actionLabel(
		formatMessage(
			shiftHeld.value && deleteHovered.value
				? commonMessages.deleteImmediatelyLabel
				: commonMessages.deleteLabel,
		),
	),
)
</script>

<template>
	<div class="flex shrink-0 items-center justify-end gap-2 transition-colors duration-200">
		<slot name="before" />
		<span
			v-if="row.synced"
			v-tooltip="syncStatusLabel"
			role="img"
			:aria-label="syncStatusLabel"
			tabindex="0"
			class="inline-flex shrink-0 cursor-help items-center justify-center rounded-full border border-solid border-brand-blue bg-highlight-blue px-2.5 py-1 text-brand-blue focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
		>
			<Link2Icon class="size-5" aria-hidden="true" />
		</span>

		<div v-if="showVersionSlot" class="flex w-9 shrink-0 items-center justify-center">
			<IconButton
				v-if="row.locked"
				v-tooltip="formatMessage(messages.frozen)"
				type="quiet"
				:label="formatMessage(messages.frozen)"
				disabled
			>
				<LockIcon class="size-5" />
			</IconButton>
			<IconButton
				v-else-if="actions.has('update') && row.hasUpdate"
				v-tooltip="actionLabel(formatMessage(commonMessages.updateButton))"
				type="quiet"
				color="green"
				:label="actionLabel(formatMessage(commonMessages.updateButton))"
				:disabled="disabled"
				class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
				@click="emit('update')"
				@mouseenter="emit('prefetch')"
				@focus="emit('prefetch')"
			>
				<DownloadIcon class="size-5" />
			</IconButton>
			<IconButton
				v-else
				v-tooltip="actionLabel(formatMessage(commonMessages.switchVersionButton))"
				type="quiet"
				:label="actionLabel(formatMessage(commonMessages.switchVersionButton))"
				:disabled="disabled"
				@click="emit('switchVersion')"
				@mouseenter="emit('prefetch')"
				@focus="emit('prefetch')"
			>
				<ArrowLeftRightIcon class="size-5" />
			</IconButton>
		</div>

		<Toggle
			v-if="showToggle"
			v-tooltip="toggleTooltip"
			:model-value="row.enabled !== false"
			:disabled="toggleDisabled"
			:aria-label="title"
			class="my-auto"
			@update:model-value="(value) => emit('toggle', value as boolean)"
		/>

		<IconButton
			v-if="showRemove"
			v-tooltip="deleteLabel"
			type="quiet"
			:label="deleteLabel"
			:disabled="disabled"
			@click="emit('remove', $event)"
			@mouseenter="deleteHovered = true"
			@mouseleave="deleteHovered = false"
			@focus="deleteHovered = true"
			@blur="deleteHovered = false"
		>
			<span class="relative size-5">
				<TrashIcon
					class="absolute inset-0 size-5 text-secondary transition-opacity duration-200"
					:class="shiftHeld && deleteHovered ? 'opacity-0' : 'opacity-100'"
				/>
				<TrashExclamationIcon
					class="absolute inset-0 size-5 text-red transition-opacity duration-200"
					:class="shiftHeld && deleteHovered ? 'opacity-100' : 'opacity-0'"
				/>
			</span>
		</IconButton>

		<slot name="after" />

		<TeleportOverflowMenu
			v-if="row.overflowOptions?.length"
			type="quiet"
			:label="formatMessage(commonMessages.moreOptionsButton)"
			:tooltip="formatMessage(commonMessages.moreOptionsButton)"
			:options="row.overflowOptions"
			:disabled="disabled"
		>
			<MoreVerticalIcon class="size-5" />
		</TeleportOverflowMenu>
	</div>
</template>
