<script setup lang="ts">
import { defineMessage, useVIntl } from '../../composables/i18n'
const { formatMessage } = useVIntl()

withDefaults(
	defineProps<{
		title?: string
		titleFor?: string
		description?: string
		optional?: boolean
	}>(),
	{
		optional: false,
	},
)

const optionalMessage = defineMessage({
	id: 'settings.form.group.optional',
	defaultMessage: '(optional)',
})
</script>

<template>
	<div class="flex flex-col gap-2">
		<div
			v-if="title || description || $slots.title || $slots.description"
			:class="description || $slots.description ? 'flex flex-col gap-1' : undefined"
		>
			<label
				v-if="title && titleFor"
				:for="titleFor"
				class="m-0 leading-normal text-contrast font-semibold"
			>
				{{ title }}
				<span v-if="optional" class="text-secondary font-normal">
					{{ formatMessage(optionalMessage) }}
				</span>
			</label>
			<p v-else-if="title" class="m-0 leading-normal text-contrast font-semibold">
				{{ title }}
				<span v-if="optional" class="text-secondary font-normal">
					{{ formatMessage(optionalMessage) }}
				</span>
			</p>
			<slot name="title" />
			<p v-if="description" class="m-0 leading-normal text-primary">
				{{ description }}
			</p>
			<slot name="description" />
		</div>
		<slot />
	</div>
</template>
