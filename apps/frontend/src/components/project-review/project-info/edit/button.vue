<template>
	<button
		v-tooltip="{ text: label, delay: 500 }"
		type="button"
		class="section-edit-button"
		:aria-label="label"
		@pointerover.stop
		@click.stop="emit('click')"
	>
		<EditIcon class="size-4" aria-hidden="true" />
	</button>
</template>

<script setup lang="ts">
import { EditIcon } from '@modrinth/assets'
import { useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { projectReviewMessages as messages } from '../../messages'

const props = defineProps<{ section: string }>()
const emit = defineEmits<{ click: [] }>()
const { formatMessage } = useVIntl()
const label = computed(() => formatMessage(messages.editSection, { section: props.section }))
</script>

<style scoped>
.section-edit-button {
	@apply inline-flex items-center bg-transparent p-0 text-inherit opacity-0 transition-opacity duration-[120ms] ease-[ease] [font:inherit];
}
.section-edit-button:hover:not(:disabled) {
	@apply text-contrast;
}
.section-edit-button:disabled {
	@apply cursor-default opacity-40;
}
.editable-review-section:hover .section-edit-button,
.section-edit-button:focus-visible {
	@apply opacity-100;
}
@media (hover: none) {
	.section-edit-button {
		@apply opacity-100;
	}
}
</style>
