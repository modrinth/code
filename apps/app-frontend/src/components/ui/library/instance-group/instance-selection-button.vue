<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { useMagicKeys } from '@vueuse/core'
import { computed } from 'vue'

defineProps<{
	selected: boolean
	compact: boolean
	isSelectionAnchor?: boolean
}>()

const emit = defineEmits<{
	(e: 'toggle', event: MouseEvent): void
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	select: { id: 'app.library.instance.select', defaultMessage: 'Select instance' },
	deselect: { id: 'app.library.instance.deselect', defaultMessage: 'Deselect instance' },
})
const keys = useMagicKeys()
const holdingShift = computed(() => keys.shift.value)
</script>

<template>
	<button
		type="button"
		class="selection-button group/selection absolute z-[2] flex size-[50px] cursor-pointer items-start pt-4 justify-center border-0 bg-transparent p-0"
		:class="compact ? '-right-1 -top-1' : 'right-2 top-1.5'"
		:aria-label="formatMessage(selected ? messages.deselect : messages.select)"
		:aria-pressed="selected"
		@click.stop="(event: MouseEvent) => emit('toggle', event)"
	>
		<span
			v-tooltip="formatMessage(selected ? messages.deselect : messages.select)"
			class="relative flex items-center justify-center rounded-full opacity-0 transition-opacity duration-200 ease-out group-hover/card:opacity-100 group-hover/selection:brightness-125"
			:class="{
				'size-[20px]': compact,
				'size-[24px]': !compact,
				'border-0 !opacity-100': selected,
				'border-2 border-solid border-primary bg-transparent': !selected,
				'[outline:3px_solid_var(--color-purple)] outline-offset-1':
					holdingShift && isSelectionAnchor,
			}"
		>
			<span v-if="selected" class="absolute inset-0 rounded-full bg-contrast" />
			<CheckIcon v-if="selected" class="relative size-4 invert [stroke-width:3]" />
		</span>
	</button>
</template>
