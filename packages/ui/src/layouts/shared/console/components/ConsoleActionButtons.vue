<template>
	<div class="flex items-center gap-1">
		<ButtonStyled type="transparent">
			<button @click="emit('clear')">
				<XIcon />
				Clear
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
			<button
				v-tooltip="shareDisabled ? shareDisabledTooltip : undefined"
				:disabled="shareDisabled"
				@click="emit('share')"
			>
				<ShareIcon />
				Share
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent">
			<button @click="emit('expand')">
				<ExpandIcon />
				Expand
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { ExpandIcon, ShareIcon, XIcon } from '@modrinth/assets'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

defineProps<{
	shareDisabled?: boolean
	shareDisabledTooltip?: string
}>()

const emit = defineEmits<{
	clear: []
	share: []
	expand: []
}>()
</script>
