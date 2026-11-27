<template>
	<Accordion
		v-if="entries?.length"
		:open-by-default="openByDefault || Boolean(filter)"
		button-class="flex w-full cursor-pointer items-start gap-1 rounded border-0 bg-transparent p-0 text-left text-inherit hover:bg-surface-3"
		@on-open="hasOpened = true"
	>
		<template #button="{ open }">
			<ChevronRightIcon
				class="mt-0.5 size-3 shrink-0 text-secondary transition-transform duration-300 motion-reduce:transition-none"
				:class="{ 'rotate-90': open }"
				aria-hidden="true"
			/>
			<code
				><span v-if="name !== undefined" class="text-blue"
					>{{ JSON.stringify(name) }}<span class="text-primary">: </span></span
				>{{ opening
				}}<span v-if="!open" class="text-secondary">…{{ closing }}{{ comma }}</span></code
			>
		</template>
		<div v-if="hasOpened || openByDefault || filter">
			<div class="ml-1.5 border-0 border-l border-solid border-surface-4 pl-3">
				<Field
					v-for="([key, child], index) in visibleEntries"
					:key="key"
					:name="isArray ? undefined : key"
					:path="childPath(key)"
					:value="child"
					:filter="filter"
					:open-by-default="isArray && entries.length === 1"
					:trailing-comma="index < visibleEntries.length - 1"
				/>
			</div>
			<code class="pl-4">{{ closing }}{{ comma }}</code>
		</div>
	</Accordion>
	<div v-else class="pl-4">
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
import { Accordion } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { fieldMatches } from './fields'

const props = defineProps<{
	name?: string
	path?: string
	value: ProjectFlagValue
	filter: string
	trailingComma?: boolean
	openByDefault?: boolean
}>()
const hasOpened = ref(false)
const isArray = computed(() => Array.isArray(props.value))
const entries = computed(() =>
	props.value !== null && typeof props.value === 'object' ? Object.entries(props.value) : undefined,
)
const visibleEntries = computed(
	() =>
		entries.value?.filter(([key, child]) => fieldMatches(childPath(key), child, props.filter)) ??
		[],
)
const opening = computed(() => (isArray.value ? '[' : '{'))
const closing = computed(() => (isArray.value ? ']' : '}'))
const comma = computed(() => (props.trailingComma ? ',' : ''))
const valueColor = computed(() => {
	if (entries.value) return undefined
	return typeof props.value === 'string' ? 'text-green' : 'text-orange'
})

function childPath(key: string) {
	return `${props.path ?? props.name}.${key}`
}
</script>
