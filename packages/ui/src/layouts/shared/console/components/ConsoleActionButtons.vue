<template>
	<div class="flex items-center gap-1">
		<ButtonStyled v-if="showClear" type="transparent">
			<button @click="emit('clear')">
				<XIcon />
				Clear
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
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
import { ContractIcon, ExpandIcon, ShareIcon, SpinnerIcon, XIcon } from '@modrinth/assets'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

defineProps<{
	showClear?: boolean
	shareDisabled?: boolean
	shareDisabledTooltip?: string
	sharing?: boolean
	fullscreen?: boolean
}>()

const emit = defineEmits<{
	clear: []
	share: []
	'toggle-fullscreen': []
}>()
</script>
