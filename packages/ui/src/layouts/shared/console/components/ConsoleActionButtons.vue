<template>
	<div class="flex items-center gap-1">
		<Button type="quiet" v-if="showClear && hasLogs"
				v-tooltip="clearDisabled ? clearDisabledTooltip : undefined"
				:disabled="clearDisabled"
				@click="emit('clear')"
			>
			<XIcon />
			Clear
		</Button>
		<Button type="quiet" color="red" v-if="showDelete"
				v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
				:disabled="deleteDisabled"
				@click="emit('delete')"
			 class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]">
			<TrashIcon />
			Delete
		</Button>
		<Button type="quiet" v-if="hasLogs"
				v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
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
import { Button } from '#ui/components/base/buttons'
import {
	ContractIcon,
	ExpandIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'


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
