<template>
	<div
		class="flex h-[750px] flex-col items-center justify-center gap-4 overflow-hidden rounded-[20px] bg-black p-4"
	>
		<div v-if="failed" class="flex flex-col items-center gap-3 text-center">
			<TriangleAlertIcon class="size-8 text-red" aria-hidden="true" />
			<p class="m-0 text-secondary">{{ formatMessage(messages.unplayable) }}</p>
		</div>
		<video
			v-else-if="kind === 'video'"
			:key="url"
			:src="url"
			class="max-h-full max-w-full rounded-lg"
			controls
			preload="metadata"
			:aria-label="name"
			@error="failed = true"
		/>
		<template v-else>
			<component :is="fileIcon.icon" class="size-16" :class="fileIcon.color" aria-hidden="true" />
			<span class="max-w-full truncate font-semibold text-contrast">{{ name }}</span>
			<audio
				:key="url"
				:src="url"
				class="w-full max-w-xl"
				controls
				preload="metadata"
				:aria-label="name"
				@error="failed = true"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { TriangleAlertIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { fileIconFor } from '../../utils'

/** Plays a video or audio file with the browser's own controls. */
const props = defineProps<{
	blob: Blob
	kind: 'video' | 'audio'
	name: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	unplayable: {
		id: 'files.media-viewer.unplayable',
		defaultMessage: "This file can't be played in the browser.",
	},
})

const fileIcon = computed(() => fileIconFor({ name: props.name, type: 'file' }))

const url = ref('')
const failed = ref(false)

watch(
	() => props.blob,
	(blob) => {
		if (url.value) URL.revokeObjectURL(url.value)
		url.value = URL.createObjectURL(blob)
		failed.value = false
	},
	{ immediate: true },
)

onBeforeUnmount(() => URL.revokeObjectURL(url.value))
</script>
