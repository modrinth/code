<script setup lang="ts">
import type { MessageDescriptor } from '@vintl/vintl'
import { useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		id?: string
		title: string | MessageDescriptor
		description?: string | MessageDescriptor
	}>(),
	{
		id: undefined,
		description: undefined,
	},
)

const formattedTitle = computed(() =>
	typeof props.title === 'string' ? props.title : formatMessage(props.title),
)
const formattedDescription = computed(() =>
	typeof props.description === 'string'
		? props.description
		: props.description
			? formatMessage(props.description)
			: undefined,
)
</script>

<template>
	<div class="mb-2">
		<label v-if="id" :for="id" class="text-lg font-extrabold text-contrast">
			{{ formattedTitle }}
		</label>
		<p v-else class="m-0 text-lg font-extrabold text-contrast">
			{{ formattedTitle }}
		</p>
		<p v-if="formattedDescription" class="text-sm m-0 text-secondary">
			{{ formattedDescription }}
		</p>
	</div>
</template>
