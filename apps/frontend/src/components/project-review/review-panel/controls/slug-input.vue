<template>
	<div class="flex flex-col gap-2">
		<Input
			ref="input"
			:model-value="modelValue"
			:placeholder="placeholder"
			:disabled="disabled"
			:aria-label="label"
			:aria-required="required"
			:aria-describedby="status ? statusId : undefined"
			autocapitalize="none"
			:spellcheck="false"
			@update:model-value="emit('update:model-value', String($event ?? ''))"
		/>
		<p
			v-if="status"
			:id="statusId"
			class="m-0 text-sm"
			:class="status.type === 'error' ? 'text-red' : 'text-secondary'"
			role="status"
		>
			{{ formatMessage(status.message) }}
		</p>
		<div v-if="suggestions.length" class="flex flex-wrap items-center gap-2">
			<span class="text-sm text-secondary">{{ formatMessage(messages.suggestions) }}</span>
			<TagItem
				v-for="suggestion in suggestions"
				:key="suggestion"
				:action="disabled ? undefined : () => emit('update:model-value', suggestion)"
				@mousedown.prevent
			>
				<CheckIcon v-if="suggestion === slug" aria-hidden="true" />
				{{ suggestion }}
			</TagItem>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { isValidProjectSlug } from '@modrinth/moderation/src/utils'
import { defineMessages, Input, injectModrinthClient, TagItem, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { refDebounced } from '@vueuse/core'
import { computed, ref, useId, watch } from 'vue'

import { useProjectSlugSuggestions } from '~/composables/project-slug-suggestions'
import { projectQueryOptions } from '~/composables/queries/project'
import { injectProjectReviewPageContext } from '~/providers/project-review'

const props = defineProps<{
	modelValue: string
	label: string
	placeholder?: string
	disabled?: boolean
	required?: boolean
}>()
const emit = defineEmits<{
	'update:model-value': [value: string]
}>()
const input = ref<InstanceType<typeof Input> | null>(null)
defineExpose({ focus: () => input.value?.focus() })

const { formatMessage } = useVIntl()
const messages = defineMessages({
	invalid: {
		id: 'project-review.controls.slug-invalid',
		defaultMessage: 'Use 3–64 letters, numbers, periods, underscores, or hyphens.',
	},
	unchanged: {
		id: 'project-review.controls.slug-unchanged',
		defaultMessage: 'Choose a URL different from the current project URL.',
	},
	available: {
		id: 'project-review.controls.slug-available',
		defaultMessage: 'This project URL is available.',
	},
	suggestions: {
		id: 'project.slug-suggestions.label',
		defaultMessage: 'Suggestions:',
	},
	taken: {
		id: 'project-review.controls.slug-taken',
		defaultMessage: 'This project URL is already taken.',
	},
	error: {
		id: 'project-review.controls.slug-error',
		defaultMessage: 'Unable to check URL availability. Try again.',
	},
})
const statusId = `${useId()}-status`
const client = injectModrinthClient()
const { project, members } = injectProjectReviewPageContext()
const slug = computed(() => props.modelValue.toLowerCase())
const ownerUsername = computed(
	() => (members.value.find((member) => member.is_owner) ?? members.value[0])?.user.username,
)
const { suggestions: availableSuggestions } = useProjectSlugSuggestions({
	title: () => project.value?.name ?? '',
	username: ownerUsername,
	currentProjectId: () => project.value?.id,
	enabled: () => !props.disabled,
})
const suggestions = computed(() =>
	availableSuggestions.value.filter(
		(suggestion) => suggestion !== project.value?.slug?.toLowerCase(),
	),
)
const isSuggestion = computed(() => suggestions.value.includes(slug.value))
const checkedSlug = refDebounced(slug, 300)
const availability = useQuery(
	computed(() => ({
		...projectQueryOptions.slugAvailability(checkedSlug.value, project.value?.id ?? '', client),
		enabled:
			!props.disabled &&
			!!project.value &&
			!isSuggestion.value &&
			isValidProjectSlug(checkedSlug.value) &&
			checkedSlug.value === slug.value &&
			checkedSlug.value !== project.value.slug?.toLowerCase(),
	})),
)
const resolvedStatus = computed(() => {
	if (!slug.value) return undefined
	if (isSuggestion.value) return { type: 'success', message: messages.available }
	if (!isValidProjectSlug(slug.value)) return { type: 'error', message: messages.invalid }
	if (slug.value === project.value?.slug?.toLowerCase())
		return { type: 'error', message: messages.unchanged }
	if (checkedSlug.value !== slug.value) return undefined
	if (availability.isError.value) return { type: 'error', message: messages.error }
	if (availability.data.value === true) return { type: 'success', message: messages.available }
	if (availability.data.value === false) return { type: 'error', message: messages.taken }
	return undefined
})
const status = ref<typeof resolvedStatus.value>()
watch(
	[resolvedStatus, slug],
	([value, currentSlug]) => {
		if (!currentSlug) status.value = undefined
		else if (value) status.value = value
	},
	{ immediate: true },
)
</script>
