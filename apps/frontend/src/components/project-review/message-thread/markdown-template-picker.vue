<template>
	<Button
		ref="trigger"
		v-tooltip="formatMessage(messages.library)"
		size="sm"
		:disabled="disabled"
		:aria-label="formatMessage(messages.library)"
		:aria-expanded="open"
		:aria-controls="listId"
		aria-haspopup="listbox"
		@click="toggle"
	>
		<LibraryIcon />
	</Button>
	<Teleport to="body">
		<div
			v-if="open"
			ref="popup"
			:style="floatingStyles"
			class="z-[100] w-72 max-w-[calc(100vw-1rem)] rounded-xl border border-solid border-surface-5 bg-surface-3 p-2 shadow-lg"
			@keydown="handleKeydown"
		>
			<form v-if="selected" class="flex flex-col gap-2" @submit.prevent="insertSelected">
				<span class="font-semibold text-contrast">{{ selected.label }}</span>
				<label v-for="field in selected.fields" :key="field.token" class="flex flex-col gap-1">
					{{ field.label }}
					<Input v-model="values[field.token]" required />
				</label>
				<Button :disabled="!fieldsComplete" @click="insertSelected">
					{{ formatMessage(messages.insert) }}
				</Button>
			</form>
			<template v-else>
				<Input
					v-if="!inlineQuery"
					ref="searchInput"
					v-model="search"
					class="mb-2 w-full"
					:placeholder="formatMessage(messages.search)"
					:aria-label="formatMessage(messages.search)"
					:aria-controls="listId"
					:aria-activedescendant="activeTemplate ? `${listId}-${active}` : undefined"
				/>
				<div
					:id="listId"
					role="listbox"
					:aria-label="formatMessage(messages.library)"
					class="max-h-72 overflow-y-auto"
				>
					<div
						v-for="(item, index) in filtered"
						:id="`${listId}-${index}`"
						:key="item.label"
						:ref="(element) => setRow(element, index)"
						role="option"
						:aria-selected="active === index"
						:aria-describedby="active === index ? tooltipId : undefined"
						class="cursor-pointer rounded-lg px-1.5 py-2 text-sm font-medium"
						:class="active === index ? 'bg-surface-5 text-contrast' : 'text-primary'"
						@mouseenter="active = index"
						@mousedown.prevent
						@click="choose(item)"
					>
						{{ item.label }}
					</div>
					<div v-if="!filtered.length" class="p-3 text-secondary">
						{{ formatMessage(messages.empty) }}
					</div>
				</div>
			</template>
		</div>
		<div
			v-if="open && !selected && activeTemplate"
			:id="tooltipId"
			ref="tooltip"
			role="tooltip"
			:style="tooltipStyles"
			class="z-[101] max-h-[min(32rem,80vh)] w-96 max-w-[calc(100vw-1rem)] overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-4 p-2 shadow-lg"
		>
			<div
				class="markdown-body text-[12px]"
				v-html="renderHighlightedString(activeTemplate.body)"
			/>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { autoUpdate, flip, offset, shift, useFloating } from '@floating-ui/vue'
import { LibraryIcon } from '@modrinth/assets'
import { Button, defineMessages, Input, useVIntl } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils/highlightjs/index'
import { onClickOutside } from '@vueuse/core'
import { computed, nextTick, ref, useId, watch } from 'vue'

import type { MarkdownTemplate, MarkdownTemplateQuery } from './markdown-templates'

const props = defineProps<{
	templates: MarkdownTemplate[]
	inlineQuery: MarkdownTemplateQuery | null
	disabled: boolean
}>()
const emit = defineEmits<{
	insert: [body: string]
	dismiss: []
}>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	library: { id: 'markdown-editor.templates.library', defaultMessage: 'Message Library' },
	search: { id: 'markdown-editor.templates.search', defaultMessage: 'Search messages…' },
	empty: { id: 'markdown-editor.templates.empty', defaultMessage: 'No matching messages.' },
	insert: { id: 'markdown-editor.templates.insert', defaultMessage: 'Insert message' },
})
const trigger = ref<InstanceType<typeof Button>>()
const popup = ref<HTMLElement>()
const tooltip = ref<HTMLElement>()
const searchInput = ref<InstanceType<typeof Input>>()
const toolbarOpen = ref(false)
const search = ref('')
const active = ref(0)
const rows = new Map<number, HTMLElement>()
const activeRow = ref<HTMLElement>()
const selected = ref<MarkdownTemplate | null>(null)
const values = ref<Record<string, string>>({})
const listId = useId()
const tooltipId = useId()
const open = computed(() => !props.disabled && (toolbarOpen.value || !!props.inlineQuery))
const query = computed(() => (props.inlineQuery?.query ?? search.value).toLowerCase().trim())
const filtered = computed(() =>
	props.templates.filter((item) => item.label.toLowerCase().includes(query.value)),
)
const activeTemplate = computed(() => filtered.value[active.value])
const reference = computed(() => props.inlineQuery?.anchor ?? trigger.value?.element ?? undefined)
const { floatingStyles } = useFloating(reference, popup, {
	placement: 'bottom-start',
	strategy: 'fixed',
	middleware: [
		offset(6),
		flip({ fallbackPlacements: ['top-start'], flipAlignment: false, padding: 8 }),
		shift({ padding: 8 }),
	],
	whileElementsMounted: autoUpdate,
})
const { floatingStyles: tooltipStyles } = useFloating(activeRow, tooltip, {
	placement: 'right-start',
	strategy: 'fixed',
	middleware: [offset(8), flip(), shift({ padding: 8 })],
	whileElementsMounted: autoUpdate,
})
const fieldsComplete = computed(() =>
	selected.value?.fields?.every((field) => values.value[field.token]?.trim()),
)
function setRow(element: unknown, index: number) {
	if (element instanceof HTMLElement) rows.set(index, element)
	else rows.delete(index)
	if (index === active.value) activeRow.value = rows.get(index)
}
watch([active, filtered], async () => {
	await nextTick()
	activeRow.value = rows.get(active.value)
	const row = activeRow.value
	const list = row?.parentElement
	if (!row || !list) return
	const rowRect = row.getBoundingClientRect()
	const listRect = list.getBoundingClientRect()
	if (rowRect.top < listRect.top) list.scrollTop += rowRect.top - listRect.top
	else if (rowRect.bottom > listRect.bottom) list.scrollTop += rowRect.bottom - listRect.bottom
})
watch(filtered, () => {
	active.value = 0
})
watch(
	() => props.inlineQuery,
	(value) => {
		if (value) toolbarOpen.value = false
	},
)
watch(
	() => props.disabled,
	(value) => {
		if (value) close()
	},
)
const triggerElement = computed(() => trigger.value?.element)
onClickOutside(
	popup,
	() => {
		if (open.value) close(false)
	},
	{ ignore: [triggerElement, tooltip] },
)
function close(restoreFocus = true) {
	toolbarOpen.value = false
	selected.value = null
	search.value = ''
	emit('dismiss')
	if (restoreFocus && !props.inlineQuery) trigger.value?.element?.focus()
}
async function toggle() {
	if (open.value) {
		close()
		return
	}
	emit('dismiss')
	toolbarOpen.value = true
	active.value = 0
	await nextTick()
	searchInput.value?.focus()
}
async function choose(item: MarkdownTemplate) {
	if (props.disabled) return
	if (item.fields?.length) {
		selected.value = item
		values.value = {}
		await nextTick()
		popup.value?.querySelector('input')?.focus()
		return
	}
	emit('insert', item.body)
	close(false)
}
function insertSelected() {
	if (!selected.value || !fieldsComplete.value || props.disabled) return
	let body = selected.value.body
	for (const field of selected.value.fields ?? [])
		body = body.replaceAll(field.token, values.value[field.token].trim())
	emit('insert', body)
	close(false)
}
function handleKeydown(event: KeyboardEvent) {
	if (!open.value || event.isComposing) return false
	if (event.key === 'Escape') {
		event.preventDefault()
		event.stopPropagation()
		close()
		return true
	}
	if (selected.value) return false
	if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
		event.preventDefault()
		event.stopPropagation()
		const count = filtered.value.length
		if (count) active.value = (active.value + (event.key === 'ArrowDown' ? 1 : -1) + count) % count
		return true
	}
	if (
		(event.key === 'Enter' || event.key === 'Tab') &&
		!event.ctrlKey &&
		!event.metaKey &&
		!event.altKey
	) {
		event.preventDefault()
		event.stopPropagation()
		if (activeTemplate.value) void choose(activeTemplate.value)
		return true
	}
	return false
}
defineExpose({
	disabled: computed(() => props.disabled),
	handleKeydown,
	listId,
	activeDescendant: computed(() =>
		open.value && !selected.value && activeTemplate.value ? `${listId}-${active.value}` : undefined,
	),
})
</script>
