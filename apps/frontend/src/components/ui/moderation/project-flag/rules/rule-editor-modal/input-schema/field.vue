<template>
	<details v-if="entries?.length" :open="openByDefault" @toggle.self="onToggle">
		<summary class="w-fit cursor-pointer list-none rounded [&::-webkit-details-marker]:hidden">
			<ChevronRightIcon
				class="field-chevron inline size-3 text-secondary"
				aria-hidden="true"
			/>
			<code
				><span v-if="name !== undefined" class="text-blue"
					>{{ JSON.stringify(name) }}<span class="text-primary">: </span></span
				>{{ opening
				}}<span class="field-preview text-secondary"
					>…{{ closing }}{{ comma }}</span
				></code
			>
		</summary>
		<div class="ml-1.5 border-0 border-l border-solid border-surface-4 pl-3">
			<Field
				v-for="([key, child], index) in entries"
				:key="key"
				:name="isArray ? undefined : key"
				:value="child"
				:open-by-default="isArray && entries.length === 1 && isOpen"
				:trailing-comma="index < entries.length - 1"
			/>
		</div>
		<code class="pl-3">{{ closing }}{{ comma }}</code>
	</details>
	<div v-else class="pl-3">
		<code
			><span v-if="name !== undefined" class="text-blue"
				>{{ JSON.stringify(name) }}<span class="text-primary">: </span></span
			><span :class="valueColor">{{ JSON.stringify(value) }}</span
			>{{ comma }}</code
		>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagValue } from '@modrinth/api-client'
import { ChevronRightIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

const props = defineProps<{
	name?: string
	value: ProjectFlagValue
	trailingComma?: boolean
	openByDefault?: boolean
}>()
const isOpen = ref(false)
const isArray = computed(() => Array.isArray(props.value))
const entries = computed(() =>
	props.value !== null && typeof props.value === 'object' ? Object.entries(props.value) : undefined,
)
const opening = computed(() => (isArray.value ? '[' : '{'))
const closing = computed(() => (isArray.value ? ']' : '}'))
const comma = computed(() => (props.trailingComma ? ',' : ''))
const valueColor = computed(() => {
	if (entries.value) return undefined
	return typeof props.value === 'string' ? 'text-green' : 'text-orange'
})

function onToggle(event: Event) {
	isOpen.value = (event.currentTarget as HTMLDetailsElement).open
}
</script>

<style scoped>
details[open] > summary > .field-chevron {
	transform: rotate(90deg);
}

details[open] > summary > code > .field-preview {
	display: none;
}
</style>
