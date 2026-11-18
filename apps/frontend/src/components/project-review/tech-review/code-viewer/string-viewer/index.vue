<script setup lang="ts">
import { CopyIcon } from '@modrinth/assets'
import {
	Button,
	Checkbox,
	Combobox,
	ContextMenu,
	injectNotificationManager,
	NewModal,
} from '@modrinth/ui'
import { hljs } from '@modrinth/utils'
import bash from 'highlight.js/lib/languages/bash'
import dos from 'highlight.js/lib/languages/dos'
import powershell from 'highlight.js/lib/languages/powershell'
import { computed, ref, useId, watch } from 'vue'

import { formatStringContents, getStringFormatParser } from './format-contents'
import { decodeStringContents } from './string-contents'

for (const [name, definition] of Object.entries({ bash, dos, powershell })) {
	if (!hljs.getLanguage(name)) hljs.registerLanguage(name, definition)
}
const languages = hljs
	.listLanguages()
	.map((value) => ({
		value,
		label:
			value === 'xml'
				? 'HTML / XML'
				: value === 'dos'
					? 'Batch'
					: (hljs.getLanguage(value)?.name ?? value),
	}))
	.sort((a, b) => a.label.localeCompare(b.label))
const languageOptions = [{ value: '', label: 'Plain text' }, ...languages]
const { addNotification } = injectNotificationManager()
const languageId = useId()
const menu = ref<InstanceType<typeof ContextMenu>>()
const modal = ref<InstanceType<typeof NewModal>>()
const content = ref<HTMLElement>()
const context = ref<{ text: string; string: boolean }>()
const raw = ref('')
const isString = ref(false)
const decode = ref(true)
const wrap = ref(true)
const language = ref('')
const formatCode = ref(true)
const formatted = ref<string>()
const formatting = ref(false)
const formatError = ref(false)
const formatParser = computed(() => getStringFormatParser(language.value))
const copyStatus = ref('')
const decoded = computed(() => (isString.value ? decodeStringContents(raw.value) : undefined))
const unformatted = computed(() =>
	decode.value && decoded.value !== undefined ? decoded.value : raw.value,
)
const displayed = computed(() => formatted.value ?? unformatted.value)

watch(
	[unformatted, formatParser, formatCode],
	async ([text, parser, enabled], _, onCleanup) => {
		formatted.value = undefined
		formatError.value = false
		formatting.value = false
		if (!enabled || !parser) return
		let current = true
		onCleanup(() => {
			current = false
		})
		formatting.value = true
		try {
			const result = await formatStringContents(text, parser)
			if (current) formatted.value = result
		} catch {
			if (current) formatError.value = true
		} finally {
			if (current) formatting.value = false
		}
	},
	{ flush: 'sync' },
)
const lines = computed(() => displayed.value.split('\n'))
const highlighted = computed(() => {
	if (!language.value) return
	try {
		const html = hljs.highlight(displayed.value, {
			language: language.value,
		}).value
		const openTags: string[] = []
		return html
			.replace(/(<span [^>]+>)|(<\/span>)|(\n)/g, (tag) => {
				if (tag === '\n') return '</span>'.repeat(openTags.length) + '\n' + openTags.join('')
				if (tag === '</span>') openTags.pop()
				else openTags.push(tag)
				return tag
			})
			.split('\n')
	} catch {
		return undefined
	}
})

async function onContextMenu(event: MouseEvent) {
	const target = event.target as HTMLElement | null
	const source = event.currentTarget as HTMLElement
	const token = target?.closest('.hljs-string')
	const selection = source.ownerDocument.getSelection()
	const selected =
		selection?.rangeCount &&
		!selection.isCollapsed &&
		source.contains(selection.anchorNode) &&
		source.contains(selection.focusNode)
			? selection.toString()
			: ''
	const text = token && source.contains(token) ? token.textContent : selected
	if (!text) return
	event.preventDefault()
	event.stopPropagation()
	context.value = {
		text,
		string: !!token,
	}
	const currentContext = context.value
	const range = token ? source.ownerDocument.createRange() : selection?.getRangeAt(0).cloneRange()
	if (token) range?.selectNodeContents(token)
	await menu.value?.open(event, [
		{
			id: 'view-contents',
			label: context.value.string ? 'View string contents' : 'View selected text',
			action: openViewer,
		},
	])
	if (context.value !== currentContext || !range || !source.isConnected) return
	selection?.removeAllRanges()
	selection?.addRange(range)
}

function detectLanguage(text: string) {
	const result = hljs.highlightAuto(
		text,
		languages.map(({ value }) => value),
	)
	if (
		result.relevance < 3 ||
		(result.secondBest && result.relevance <= result.secondBest.relevance)
	)
		return ''
	return result.language ?? ''
}

function openViewer(event: MouseEvent) {
	if (!context.value) return
	formatCode.value = true
	raw.value = context.value.text
	isString.value = context.value.string
	decode.value = true
	language.value = detectLanguage(unformatted.value)
	copyStatus.value = ''
	menu.value?.close()
	modal.value?.show(event)
}

async function copy() {
	try {
		const clipboard = content.value?.ownerDocument.defaultView?.navigator.clipboard
		if (!clipboard) throw new Error('Clipboard unavailable')
		await clipboard.writeText(displayed.value)
		copyStatus.value = 'Copied'
	} catch {
		addNotification({
			title: 'Could not copy contents',
			text: 'Select the text and copy it manually.',
			type: 'error',
		})
	}
}

defineExpose({ onContextMenu })
</script>

<template>
	<ContextMenu ref="menu" label="Code actions" />
	<NewModal
		ref="modal"
		:header="isString ? 'String contents' : 'Selected text'"
		width="60rem"
		max-width="calc(100vw - 2rem)"
		:animate-resize="false"
	>
		<div class="flex min-w-0 flex-col gap-3">
			<div class="flex flex-wrap items-center gap-5">
				<label :for="languageId" class="font-semibold text-contrast">Language</label>
				<Combobox
					v-model="language"
					:options="languageOptions"
					:search-input-attrs="{ id: languageId, 'aria-label': 'Language' }"
					searchable
					select-search-text-on-focus
					search-placeholder="Search languages..."
					class="!w-48 max-w-full"
				/>
				<Checkbox v-if="isString" v-model="decode" label="Decode escapes" />
				<Checkbox v-model="wrap" label="Wrap lines" />
				<Checkbox v-model="formatCode" label="Format code" :disabled="!formatParser" />
				<Button class="ml-auto" :disabled="formatting" @click="copy"
					><CopyIcon aria-hidden="true" />Copy</Button
				>
				<span role="status" class="text-sm text-secondary">{{ copyStatus }}</span>
			</div>
			<p v-if="isString && decode && decoded === undefined" class="m-0 text-sm text-secondary">
				This string could not be decoded. Showing the original text.
			</p>
			<p v-if="formatting" role="status" class="m-0 text-sm text-secondary">Formatting...</p>
			<p v-else-if="formatError" role="status" class="m-0 text-sm text-secondary">
				Could not format this text. Showing the original contents.
			</p>
			<div
				ref="content"
				class="max-h-[60vh] overflow-auto rounded-lg bg-surface-1 p-3"
				tabindex="0"
				aria-label="String contents"
			>
				<div
					class="font-mono text-[13px] font-normal leading-5 [tab-size:2]"
					:class="wrap ? 'min-w-0' : 'w-max min-w-full'"
				>
					<div v-for="(line, index) in lines" :key="index" class="flex items-stretch">
						<span
							aria-hidden="true"
							class="w-12 shrink-0 select-none border-0 border-r border-solid border-surface-5 pr-3 text-right text-secondary"
							>{{ index + 1 }}</span
						>
						<div
							class="min-w-0 flex-1 pl-3 text-primary"
							:class="wrap ? 'whitespace-pre-wrap [overflow-wrap:anywhere]' : 'whitespace-pre'"
						>
							<span v-if="highlighted" v-html="highlighted[index] || ' '" /><span v-else>{{
								line || ' '
							}}</span>
						</div>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
