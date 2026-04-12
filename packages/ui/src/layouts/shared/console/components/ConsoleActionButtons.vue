<template>
	<div class="flex items-center gap-1">
		<ButtonStyled v-if="showClear && hasLogs" type="transparent">
			<button @click="emit('clear')">
				<XIcon />
				Clear
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="showDelete" type="transparent" hover-color-fill="background" color="red">
			<button
				v-tooltip="deleteDisabled ? deleteDisabledTooltip : undefined"
				:disabled="deleteDisabled"
				@click="emit('delete')"
			>
				<TrashIcon />
				Delete
			</button>
		</ButtonStyled>
		<ButtonStyled v-if="hasLogs" type="transparent">
			<button
				v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
				:disabled="shareDisabled || sharing"
				@click="emit('share')"
			>
				<SpinnerIcon v-if="sharing" class="animate-spin" />
				<ShareIcon v-else />
				Share
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
			<button @click="emit('toggle-fullscreen')">
				<ContractIcon v-if="fullscreen" />
				<ExpandIcon v-else />
				{{ fullscreen ? 'Collapse' : 'Expand' }}
			</button>
		</ButtonStyled>
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

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

defineProps<{
	showClear?: boolean
	hasLogs?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
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
