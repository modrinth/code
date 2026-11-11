<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, FileScanIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { formatKeybind, moderationSettings } from '@modrinth/moderation'
import { IconButton } from '@modrinth/ui'
import { computed, onMounted, ref, useId } from 'vue'

import { useModerationKeybinds, useModerationSettings } from '~/composables/moderation'

import type { SourceMatchMode, SourceSearchOptions } from './source-search'

const props = defineProps<{
	mode: SourceMatchMode
	hasTraceMatches: boolean
	current: number
	total: number
	error: string
}>()
const query = defineModel<string>('query', { required: true })
const options = defineModel<SourceSearchOptions>('options', { required: true })
const emit = defineEmits<{
	search: []
	traces: []
	close: []
	navigate: [direction: number]
}>()
const root = ref<HTMLElement>()
const input = ref<HTMLInputElement>()
const errorId = useId()
const keybinds = useModerationKeybinds()
const settings = useModerationSettings()
const bindings = computed(() => new Map([...keybinds.value]))
const isSearchMode = computed(() => props.mode === 'search')
const counterText = computed(() =>
	props.total && (!isSearchMode.value || query.value)
		? `${props.current + 1} of ${props.total}`
		: 'No results',
)
const modeControls = computed(() =>
	isSearchMode.value
		? {
				previousKeybind: 'tech-review-find-previous',
				nextKeybind: 'tech-review-find-next',
				closeLabel: 'Close find',
			}
		: {
				previousKeybind: 'tech-review-previous-highlight',
				nextKeybind: 'tech-review-next-highlight',
				closeLabel: 'Hide trace matches',
			},
)
const isMac = ref(false)
onMounted(() => {
	isMac.value = (root.value?.ownerDocument.defaultView?.navigator.platform ?? '')
		.toUpperCase()
		.includes('MAC')
})
const searchOptions = [
	{ key: 'matchCase', text: 'Aa', label: 'Match case', keybind: 'tech-review-find-match-case' },
	{
		key: 'wholeWord',
		text: 'ab',
		label: 'Match whole word',
		keybind: 'tech-review-find-whole-word',
	},
	{ key: 'regex', text: '.*', label: 'Use regular expression', keybind: 'tech-review-find-regex' },
] as const

function tooltip(label: string, id: string) {
	if (!settings.value.get(moderationSettings.General.ShowShortcutKeybindHints)) return label
	const shortcuts = bindings.value
		.get(id)
		?.keybind.map((definition) => formatKeybind(definition, isMac.value))
	return shortcuts?.length ? `${label} (${shortcuts.join(' / ')})` : label
}

defineExpose({
	contains: (target: Node | null) => !!root.value?.contains(target),
	isInput: (target: Node | null) => target === input.value,
	focus: () => {
		input.value?.focus()
		input.value?.select()
	},
})
</script>

<template>
	<div
		ref="root"
		role="search"
		aria-label="Find in source file"
		class="absolute right-5 top-2 z-30 flex max-w-[calc(100%_-_2rem)] flex-wrap items-center gap-0.5 rounded-xl border border-solid border-surface-5 bg-surface-3 p-1 shadow-lg"
		:class="isSearchMode ? 'w-[28rem]' : 'w-max'"
	>
		<template v-if="mode === 'closed'">
			<IconButton
				v-tooltip="tooltip('Find in file', 'tech-review-find')"
				type="quiet"
				size="xs"
				label="Find in file"
				@click="emit('search')"
			>
				<SearchIcon />
			</IconButton>
			<IconButton
				v-if="hasTraceMatches"
				v-tooltip="tooltip('Show trace matches', 'tech-review-show-highlights')"
				type="quiet"
				size="xs"
				label="Show trace matches"
				@click="emit('traces')"
			>
				<FileScanIcon />
			</IconButton>
		</template>
		<template v-else>
			<template v-if="isSearchMode">
				<div class="flex h-7 min-w-40 flex-1 items-center gap-px rounded-lg bg-surface-3 px-0">
					<SearchIcon class="ml-1 size-4 shrink-0 text-secondary" aria-hidden="true" />
					<input
						ref="input"
						v-model="query"
						type="text"
						class="h-7 min-w-0 flex-1 rounded-lg border-0 bg-transparent px-1.5 py-0 text-sm text-primary !outline-none placeholder:text-secondary focus:!shadow-none focus-visible:!outline-none"
						placeholder="Find in file"
						aria-label="Find in file"
						:aria-invalid="!!error"
						:aria-describedby="error ? errorId : undefined"
						spellcheck="false"
					/>
					<IconButton
						v-for="option in searchOptions"
						:key="option.key"
						v-tooltip="tooltip(option.label, option.keybind)"
						type="quiet"
						size="xs"
						:circular="false"
						class="!size-[22px] shrink-0 !p-0 font-sans !text-[11px] font-semibold"
						:class="{ '!bg-surface-4': options[option.key] }"
						:label="option.label"
						:aria-pressed="options[option.key]"
						@click="options = { ...options, [option.key]: !options[option.key] }"
					>
						<span :class="{ underline: option.key === 'wholeWord' }">{{ option.text }}</span>
					</IconButton>
				</div>
				<span
					aria-hidden="true"
					class="-my-1 ml-0.5 mr-1.5 w-px shrink-0 self-stretch bg-surface-5"
				/>
			</template>
			<FileScanIcon v-else aria-hidden="true" class="ml-1 mr-1 size-4 shrink-0 text-secondary" />
			<span
				role="status"
				class="grid whitespace-nowrap text-xs tabular-nums text-secondary"
				:class="{ 'mx-1.5': !isSearchMode }"
			>
				<span v-if="isSearchMode" aria-hidden="true" class="invisible col-start-1 row-start-1"
					>No results</span
				>
				<span class="col-start-1 row-start-1">{{ counterText }}</span>
			</span>
			<IconButton
				v-tooltip="tooltip('Previous match', modeControls.previousKeybind)"
				type="quiet"
				size="xs"
				label="Previous match"
				:disabled="!total"
				@click="emit('navigate', -1)"
			>
				<ArrowUpIcon />
			</IconButton>
			<IconButton
				v-tooltip="tooltip('Next match', modeControls.nextKeybind)"
				type="quiet"
				size="xs"
				label="Next match"
				:disabled="!total"
				@click="emit('navigate', 1)"
			>
				<ArrowDownIcon />
			</IconButton>
			<IconButton
				v-tooltip="tooltip(modeControls.closeLabel, 'tech-review-find-close')"
				type="quiet"
				size="xs"
				:label="modeControls.closeLabel"
				@click="emit('close')"
			>
				<XIcon />
			</IconButton>
			<p v-if="error" :id="errorId" role="alert" class="m-0 w-full text-xs text-red">{{ error }}</p>
		</template>
	</div>
</template>
