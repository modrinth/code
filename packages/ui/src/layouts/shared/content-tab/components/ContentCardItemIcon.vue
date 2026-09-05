<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed, onScopeDispose, ref, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'

import type { ContentCardEmbeddedIcon } from '../types'

const props = defineProps<{
	src?: string | null
	fallbackUrl?: string | null
	embeddedIcon?: ContentCardEmbeddedIcon
	alt: string
	tintBy?: string
}>()

const queryKey = computed(
	() => props.embeddedIcon?.queryKey ?? (['content', 'embedded-icon', 'disabled'] as const),
)

const embeddedIconQuery = useQuery({
	queryKey,
	queryFn: () => {
		if (!props.embeddedIcon) throw new Error('Missing embedded icon request')
		return props.embeddedIcon.queryFn()
	},
	enabled: computed(
		() => typeof window !== 'undefined' && !props.src && props.embeddedIcon !== undefined,
	),
	staleTime: Infinity,
})

const embeddedIconUrl = ref<string>()
let pendingIconUrl: string | undefined
let validationId = 0

watch(
	() => embeddedIconQuery.data.value,
	(blob) => {
		validationId += 1
		const currentValidationId = validationId
		if (embeddedIconUrl.value) URL.revokeObjectURL(embeddedIconUrl.value)
		if (pendingIconUrl) URL.revokeObjectURL(pendingIconUrl)
		embeddedIconUrl.value = undefined
		pendingIconUrl = undefined
		if (!blob) return

		const candidateUrl = URL.createObjectURL(blob)
		pendingIconUrl = candidateUrl
		const image = new Image()
		image.onload = () => {
			if (currentValidationId !== validationId) return
			pendingIconUrl = undefined
			embeddedIconUrl.value = candidateUrl
		}
		image.onerror = () => {
			if (currentValidationId !== validationId) return
			pendingIconUrl = undefined
			URL.revokeObjectURL(candidateUrl)
		}
		image.src = candidateUrl
	},
	{ immediate: true },
)

onScopeDispose(() => {
	validationId += 1
	if (embeddedIconUrl.value) URL.revokeObjectURL(embeddedIconUrl.value)
	if (pendingIconUrl) URL.revokeObjectURL(pendingIconUrl)
})

const resolvedSrc = computed(
	() =>
		props.src ??
		embeddedIconUrl.value ??
		props.embeddedIcon?.fallbackUrl ??
		props.fallbackUrl ??
		undefined,
)
</script>

<template>
	<Avatar
		:src="resolvedSrc"
		:alt="alt"
		:tint-by="tintBy"
		size="3rem"
		no-shadow
		class="rounded-2xl border border-surface-5"
	/>
</template>
