<template>
	<div class="flex items-center gap-0.5">
		<ButtonStyled type="transparent" circular>
			<button @click="emit('clear')">
				<XIcon />
				Clear
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular>
			<button @click="handleCopy">
				<CopyIcon />
				{{ copyLabel }}
			</button>
		</ButtonStyled>
		<ButtonStyled type="transparent" circular>
			<button :disabled="shareDisabled" @click="emit('share')">
				<ShareIcon />
				Share
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { CopyIcon, ShareIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

defineProps<{
	shareDisabled?: boolean
}>()

const emit = defineEmits<{
	clear: []
	copy: []
	share: []
}>()

const copyLabel = ref('Copy')
let copyTimeout: ReturnType<typeof setTimeout> | null = null

function handleCopy() {
	emit('copy')
	copyLabel.value = 'Copied!'
	if (copyTimeout) clearTimeout(copyTimeout)
	copyTimeout = setTimeout(() => {
		copyLabel.value = 'Copy'
	}, 2000)
}
</script>
