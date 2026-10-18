<template>
	<button
		type="button"
		class="flex w-full cursor-pointer items-center justify-center gap-1 border-0 border-t border-dashed border-surface-4 bg-transparent py-1 text-xs font-medium text-secondary transition-colors hover:bg-surface-3 hover:text-contrast focus-visible:outline-offset-[-2px] motion-reduce:transition-none"
		:aria-expanded="open"
		:aria-controls="controls"
		@click="emit('update:open', !open)"
	>
		<ChevronDownIcon
			class="size-4 transition-transform motion-reduce:transition-none"
			:class="{ 'rotate-180': open }"
			aria-hidden="true"
		/>
		{{ formatMessage(open ? messages.hideDetails : messages.showDetails) }}
	</button>
</template>

<script setup lang="ts">
import { ChevronDownIcon } from '@modrinth/assets'

import { useVIntl } from '#ui/composables/i18n'

import { sectionMessages as messages } from './messages'

/** The tab along the bottom of a section that pulls its details out, or tucks them away. */
defineProps<{
	open: boolean
	/** The id of the details it shows and hides. */
	controls: string
}>()

const emit = defineEmits<{
	'update:open': [open: boolean]
}>()

const { formatMessage } = useVIntl()
</script>
