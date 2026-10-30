<script setup lang="ts">
import { computed, getCurrentInstance } from 'vue'

const TraceData = getCurrentInstance()!.type
const props = defineProps<{
	value: unknown
	matchSource?: boolean
	canHighlight?: (value: string) => boolean
}>()
const emit = defineEmits<{
	select: [value: string]
}>()

const entries = computed(() =>
	props.value && typeof props.value === 'object' && !Array.isArray(props.value)
		? Object.entries(props.value).sort(([a], [b]) => a.localeCompare(b))
		: [],
)
const url = computed(() => {
	if (typeof props.value !== 'string') return undefined
	try {
		const parsed = new URL(props.value)
		return parsed.protocol === 'http:' || parsed.protocol === 'https:' ? parsed.href : undefined
	} catch {
		return undefined
	}
})
const interactive = computed(
	() =>
		typeof props.value === 'string' &&
		Boolean(props.matchSource || url.value) &&
		Boolean(props.canHighlight?.(props.value)),
)

function select() {
	if (interactive.value) emit('select', String(props.value))
}

function onUrlClick(event: MouseEvent) {
	if (
		!interactive.value ||
		event.button !== 0 ||
		event.ctrlKey ||
		event.metaKey ||
		event.shiftKey ||
		event.altKey
	)
		return
	event.preventDefault()
	select()
}

const text = computed(() => {
	if (props.value === null) return 'null'
	if (props.value === undefined) return '—'
	if (props.value === '') return '(empty)'
	return String(props.value)
})
</script>

<template>
	<ul
		v-if="Array.isArray(value) && value.length"
		class="m-0 list-disc space-y-1 pl-5 marker:text-secondary"
	>
		<li v-for="(item, index) in value" :key="index" class="min-w-0 pl-0">
			<TraceData
				:value="item"
				match-source
				:can-highlight="canHighlight"
				@select="emit('select', $event)"
			/>
		</li>
	</ul>
	<span v-else-if="Array.isArray(value)" class="text-xs text-secondary">Empty list</span>
	<dl
		v-else-if="value !== null && typeof value === 'object' && entries.length"
		class="m-0 grid min-w-0 grid-cols-[fit-content(40%)_minmax(0,1fr)] items-start gap-x-3 gap-y-2 text-xs leading-relaxed"
	>
		<template v-for="[key, item] in entries" :key="key">
			<dt class="max-w-40 truncate whitespace-nowrap font-medium text-secondary" :title="key">
				{{ key }}
			</dt>
			<dd class="m-0 min-w-0">
				<TraceData
					:value="item"
					:match-source="matchSource || key.toLowerCase() === 'url'"
					:can-highlight="canHighlight"
					@select="emit('select', $event)"
				/>
			</dd>
		</template>
	</dl>
	<span v-else-if="value !== null && typeof value === 'object'" class="text-xs text-secondary"
		>No fields</span
	>
	<a
		v-else-if="url"
		:href="url"
		target="_blank"
		rel="noopener noreferrer"
		class="rounded-sm font-mono text-xs leading-relaxed text-brand-blue [overflow-wrap:anywhere] hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:title="interactive ? 'Go to source (Ctrl/Cmd-click to open link)' : undefined"
		@click="onUrlClick"
		>{{ text }}</a
	>
	<button
		v-else-if="interactive"
		type="button"
		title="Go to source"
		class="cursor-pointer rounded-sm border-0 bg-transparent p-0 text-left font-mono text-xs leading-relaxed text-brand-blue [overflow-wrap:anywhere] hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		@click="select"
	>
		{{ text }}
	</button>
	<span
		v-else
		class="font-mono text-xs leading-relaxed [overflow-wrap:anywhere]"
		:class="{
			'text-green': value === true,
			'text-red': value === false,
			'text-secondary': value === null || value === undefined,
			'text-contrast': value !== null && value !== undefined && typeof value !== 'boolean',
		}"
		>{{ text }}</span
	>
</template>
