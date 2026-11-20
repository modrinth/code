<template>
	<div
		class="relative size-12 shrink-0 transition-[width,margin-right,opacity] duration-200 ease-in-out motion-reduce:transition-none"
		:class="{ 'pointer-events-none -mr-4 w-0 opacity-0': hidden }"
	>
		<IconButton
			v-tooltip="label"
			size="xl"
			:label="label"
			class="absolute right-0 top-0"
			@click="emit('click')"
		>
			<LeftArrowIcon aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import { LeftArrowIcon } from '@modrinth/assets'

import { IconButton } from '#ui/components/base/buttons'

defineProps<{
	label: string
	hidden?: boolean
}>()

const emit = defineEmits<{
	click: []
}>()
</script>
