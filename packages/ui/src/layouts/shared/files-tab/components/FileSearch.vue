<template>
	<div ref="root" class="relative" @focusout="handleFocusOut">
		<Input
			v-model="query"
			:icon="SearchIcon"
			type="search"
			name="file-search"
			autocomplete="off"
			:placeholder="formatMessage(messages.placeholder)"
			size="medium"
			wrapper-class="w-full"
			role="combobox"
			aria-autocomplete="list"
			:aria-expanded="open"
			:aria-controls="listId"
			:aria-activedescendant="open && results.length > 0 ? optionId(highlighted) : undefined"
			@focus="focused = true"
			@keydown="handleKeydown"
		/>
		<div
			v-if="open"
			class="absolute inset-x-0 top-full z-40 mt-1 flex flex-col overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-3 shadow-lg"
		>
			<ul :id="listId" role="listbox" class="m-0 max-h-80 list-none overflow-y-auto p-1">
				<li
					v-for="(result, index) in results"
					:id="optionId(index)"
					:key="result.entry.path"
					role="option"
					:aria-selected="index === highlighted"
					class="flex cursor-pointer items-center gap-2 rounded-lg px-2 py-1.5"
					:class="index === highlighted ? 'bg-surface-5 text-contrast' : 'text-primary'"
					@mousedown.prevent="select(result.entry)"
					@mouseenter="highlighted = index"
				>
					<component
						:is="result.icon.icon"
						class="size-4 shrink-0"
						:class="ui.coloredIcons.value ? result.icon.color : 'text-secondary'"
						aria-hidden="true"
					/>
					<span class="truncate text-sm font-medium">{{ result.entry.name }}</span>
					<span class="ml-auto shrink truncate pl-2 text-xs text-secondary">
						{{ result.parent }}
					</span>
				</li>
				<li v-if="results.length === 0" class="px-2 py-1.5 text-sm text-secondary">
					{{ formatMessage(messages.noResults) }}
				</li>
			</ul>
			<p
				v-if="loadedOnlyNotice"
				class="m-0 border-0 border-t border-solid border-surface-5 px-3 py-1.5 text-xs text-secondary"
			>
				{{ formatMessage(messages.loadedOnly) }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import Fuse from 'fuse.js'
import { computed, nextTick, ref, useId, watch } from 'vue'

import Input from '#ui/components/base/inputs/Input.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { canOpenInFileEditor } from '#ui/utils/file-extensions'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FileItem } from '../types'
import { fileIconFor, parentInfoFrom } from '../utils'

const MAX_RESULTS = 30

withDefaults(
	defineProps<{
		/** Notes that results only cover folders opened so far, for hosts that load lazily. */
		loadedOnlyNotice?: boolean
	}>(),
	{ loadedOnlyNotice: true },
)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	placeholder: {
		id: 'files.search.placeholder',
		defaultMessage: 'Go to file or folder',
	},
	noResults: {
		id: 'files.search.no-results',
		defaultMessage: 'No matching files or folders',
	},
	loadedOnly: {
		id: 'files.search.loaded-only',
		defaultMessage: 'Searching folders that have been opened',
	},
})

const ui = injectFileBrowserUI()

const root = ref<HTMLElement | null>(null)
const query = ref('')
const focused = ref(false)
const highlighted = ref(0)

const listId = `file-search-${useId()}`
const optionId = (index: number) => `${listId}-${index}`

const fuse = computed(
	() =>
		new Fuse(ui.directoryTree.loadedEntries?.value ?? [], {
			keys: [
				{ name: 'name', weight: 2 },
				{ name: 'path', weight: 1 },
			],
			threshold: 0.4,
			ignoreLocation: true,
		}),
)

const results = computed(() => {
	const trimmed = query.value.trim()
	if (!trimmed) return []
	return fuse.value.search(trimmed, { limit: MAX_RESULTS }).map(({ item }) => ({
		entry: item,
		icon: fileIconFor(item),
		parent: parentInfoFrom(item).path,
	}))
})

const open = computed(() => focused.value && query.value.trim().length > 0)

watch(query, () => (highlighted.value = 0))

/** Opens directories and editable files; any other file reveals its folder instead. */
function select(entry: FileItem) {
	const openable = entry.type === 'directory' || canOpenInFileEditor(entry.name)
	ui.navigateTo(openable ? entry : parentInfoFrom(entry))
	query.value = ''
}

function moveHighlight(delta: number) {
	const count = results.value.length
	if (count === 0) return
	highlighted.value = (highlighted.value + delta + count) % count
	nextTick(() =>
		document.getElementById(optionId(highlighted.value))?.scrollIntoView({ block: 'nearest' }),
	)
}

function handleKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'ArrowDown':
			event.preventDefault()
			moveHighlight(1)
			break
		case 'ArrowUp':
			event.preventDefault()
			moveHighlight(-1)
			break
		case 'Enter': {
			const result = results.value[highlighted.value]
			if (!result) return
			event.preventDefault()
			select(result.entry)
			break
		}
		case 'Escape':
			query.value = ''
			break
	}
}

function handleFocusOut(event: FocusEvent) {
	if (!root.value?.contains(event.relatedTarget as Node | null)) focused.value = false
}
</script>
