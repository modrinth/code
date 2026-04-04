<template>
	<div class="flex items-center gap-0.5">
		<ButtonStyled type="transparent" circular hover-color-fill="none">
			<button @click="emit('clear')">
				<XIcon />
				Clear
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular hover-color-fill="none">
			<button :disabled="shareDisabled" @click="emit('share')">
				<ShareIcon />
				Share
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular hover-color-fill="none">
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
}>()

const emit = defineEmits<{
	clear: []
	share: []
	expand: []
}>()
</script>
