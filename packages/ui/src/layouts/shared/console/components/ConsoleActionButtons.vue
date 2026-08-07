<template>
	<div class="flex items-center gap-1">
		<Button
			v-if="showClear && hasLogs"
			v-tooltip="clearDisabled ? clearDisabledTooltip : undefined"
			type="quiet"
			:disabled="clearDisabled"
			@click="emit('clear')"
		>
			<XIcon />
			Clear
		</Button>
		<Button
			v-if="showDelete"
			v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
			type="quiet"
			color="red"
			:disabled="deleteDisabled"
			class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
			@click="emit('delete')"
		>
			<TrashIcon />
			Delete
		</Button>
		<Button
			v-if="hasLogs"
			v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
			type="quiet"
			:disabled="shareDisabled || sharing"
			@click="emit('share')"
		>
			<SpinnerIcon v-if="sharing" class="animate-spin" />
			<ShareIcon v-else />
			Share
		</Button>
		<Button type="quiet" @click="emit('toggle-fullscreen')">
			<ContractIcon v-if="fullscreen" />
			<ExpandIcon v-else />
			{{ fullscreen ? 'Collapse' : 'Expand' }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import {
	ContractIcon,
	ExpandIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'

import { Button } from '#ui/components/base/buttons'

defineProps<{
	showClear?: boolean
	hasLogs?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
	clearDisabled?: boolean
	clearDisabledTooltip?: string
	showDelete?: boolean
	deleteDisabled?: boolean
	deleteDisabledTooltip?: string
}>()

const emit = defineEmits<{
	clear: []
	share: []
	'toggle-fullscreen': []
	delete: []
}>()
</script>
