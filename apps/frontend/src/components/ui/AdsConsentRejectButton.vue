<template>
	<Tooltip
		class="-m-1.5 inline-flex"
		theme="tooltip"
		:triggers="['hover', 'focus']"
		:delay="{ show: 0, hide: 0 }"
	>
		<ButtonStyled circular type="transparent">
			<button type="button" class="!shadow-none" :aria-label="label" @click="action()">
				<XIcon />
			</button>
		</ButtonStyled>
		<template #popper>{{ label }}</template>
	</Tooltip>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { Tooltip } from 'floating-vue'

defineProps<{
	label: string
	action: () => void | Promise<void>
}>()
</script>
