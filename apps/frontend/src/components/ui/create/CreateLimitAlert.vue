<template>
	<Admonition
		v-if="shouldShowAlert"
		type="warning"
		:header="`Approaching ${typeDisplayName} limit`"
		class="mb-4"
	>
		<div class="flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<span class="text-sm">
					{{ current }} of {{ max }} {{ typeDisplayName }}{{ max !== 1 ? 's' : '' }} created
				</span>
				<span class="text-lg font-semibold">{{ percentage }}%</span>
			</div>
			<div class="h-3 w-full rounded-full border-solid border-highlight-orange bg-highlight-orange">
				<div
					class="h-full rounded-full bg-brand-orange transition-all duration-300"
					:style="`width: ${percentage}%`"
				/>
			</div>
			<p class="m-0 text-sm text-contrast">
				Please contact support to increase your
				{{ typeDisplayName }}s limit.
			</p>
		</div>
	</Admonition>
</template>

<script setup lang="ts">
import { Admonition } from '@modrinth/ui'
import { computed } from 'vue'

interface LimitsResponse {
	projects: { current: number; max: number } | null
	organizations: { current: number; max: number } | null
	collections: { current: number; max: number } | null
}

const props = defineProps<{
	type: 'project' | 'org' | 'collection'
}>()

// const { data: limits } = await useAsyncData('limits', () =>
// 	useBaseFetch('limits', {
// 		apiVersion: 3,
// 	}),
// )

const limits = ref({
	projects: { current: 8, max: 10 },
	organizations: { current: 3, max: 4 },
	collections: { current: 15, max: 20 },
})

const limitKey = computed(() => {
	switch (props.type) {
		case 'project':
			return 'projects'
		case 'org':
			return 'organizations'
		case 'collection':
			return 'collections'
		default:
			return 'projects'
	}
})

const typeDisplayName = computed(() => {
	switch (props.type) {
		case 'project':
			return 'project'
		case 'org':
			return 'organization'
		case 'collection':
			return 'collection'
		default:
			return 'project'
	}
})

const current = computed(() => limits.value?.[limitKey.value]?.current ?? 0)
const max = computed(() => limits.value?.[limitKey.value]?.max ?? null)
const percentage = computed(() => (max.value ? Math.round((current.value / max.value) * 100) : 0))
const shouldShowAlert = computed(() => max.value !== null && percentage.value >= 75)
</script>
