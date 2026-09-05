<template>
	<Transition name="slug-suggestions">
		<div v-if="visible && hasSuggestions" class="mt-2 grid grid-rows-[1fr]">
			<div class="flex min-h-0 flex-wrap items-center gap-2 overflow-hidden">
				<span class="text-sm text-secondary">{{ formatMessage(messages.label) }}</span>
				<TagItem
					v-for="suggestion in suggestions"
					:key="suggestion"
					:action="() => emit('select', suggestion)"
					@mousedown.prevent
				>
					<CheckIcon v-if="suggestion === selected" aria-hidden="true" />
					{{ suggestion }}
				</TagItem>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { defineMessages, TagItem, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{
	selected: string
	suggestions: string[]
	visible: boolean
}>()

const emit = defineEmits<{
	select: [suggestion: string]
}>()

const hasSuggestions = computed(() =>
	props.suggestions.some((suggestion) => suggestion !== props.selected),
)

const { formatMessage } = useVIntl()
const messages = defineMessages({
	label: {
		id: 'project.slug-suggestions.label',
		defaultMessage: 'Suggestions:',
	},
})
</script>

<style scoped>
.slug-suggestions-enter-active,
.slug-suggestions-leave-active {
	transition:
		grid-template-rows 150ms ease,
		opacity 150ms ease,
		transform 150ms ease;
}

.slug-suggestions-enter-from,
.slug-suggestions-leave-to {
	grid-template-rows: 0fr;
	opacity: 0;
	transform: translateY(-0.25rem);
}
</style>
