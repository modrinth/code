<template>
	<div class="flex items-center gap-1">
		<Button
			v-if="showClear && hasLogs"
			v-tooltip="clearDisabled ? clearDisabledTooltip : undefined"
			type="quiet"
			:disabled="clearDisabled"
			@click="emit('clear')"
		>
			<XIcon aria-hidden="true" />
			Clear
		</Button>
		<Button
			v-if="showDelete"
			v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
			type="quiet"
			color="red"
			:disabled="deleteDisabled"
			@click="emit('delete')"
		>
			<TrashIcon aria-hidden="true" />
			Delete
		</Button>
		<Button
			v-if="hasLogs"
			v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
			type="quiet"
			:loading="sharing"
			:disabled="shareDisabled"
			@click="emit('share')"
		>
			<SpinnerIcon v-if="sharing" class="animate-spin" aria-hidden="true" />
			<ShareIcon v-else aria-hidden="true" />
			Share
		</Button>
		<Button type="quiet" @click="emit('toggle-fullscreen')">
			<ContractIcon v-if="fullscreen" aria-hidden="true" />
			<ExpandIcon v-else aria-hidden="true" />
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

import Button from '#ui/components/base/buttons/Button.vue'

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
