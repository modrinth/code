<template>
	<Transition name="find">
		<div
			v-if="isFindOpen && !isEditingImage"
			class="absolute right-3 top-3 z-10 flex flex-col gap-1 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-1.5 shadow-lg"
			@keydown.escape.stop="close"
		>
			<!-- Find row -->
			<div class="flex items-center gap-1">
				<IconButton
					:label="formatMessage(messages.toggleReplace)"
					type="quiet"
					:disabled="props.readonly"
					@click="toggleReplace"
				>
					<ChevronRightIcon
						aria-hidden="true"
						class="transition-transform duration-150"
						:class="{ 'rotate-90': isReplaceOpen }"
					/>
				</IconButton>
				<div
					@keydown.enter.exact.prevent.stop="emit('findNext')"
					@keydown.shift.enter.prevent.stop="emit('findPrevious')"
				>
					<StyledInput
						ref="findInputRef"
						:model-value="findQuery"
						type="search"
						size="small"
						autocomplete="off"
						:placeholder="formatMessage(messages.findInFile)"
						wrapper-class="w-44"
						@update:model-value="emit('update:findQuery', $event as string)"
					/>
				</div>
				<span class="min-w-[6rem] px-1 text-sm text-secondary tabular-nums">
					{{
						findMatchCount > 0
							? formatMessage(messages.matchCount, {
									current: currentFindMatch,
									total: findMatchCount,
								})
							: findQuery
								? formatMessage(messages.noResults)
								: ''
					}}
				</span>
				<IconButton
					:label="formatMessage(messages.previousMatch)"
					type="quiet"
					:disabled="findMatchCount === 0"
					@click="emit('findPrevious')"
				>
					<ChevronUpIcon aria-hidden="true" />
				</IconButton>
				<IconButton
					:label="formatMessage(messages.nextMatch)"
					type="quiet"
					:disabled="findMatchCount === 0"
					@click="emit('findNext')"
				>
					<ChevronDownIcon aria-hidden="true" />
				</IconButton>
				<div class="mx-0.5 h-4 w-px bg-surface-5" />
				<IconButton
					:label="formatMessage(messages.closeFind)"
					type="quiet"
					@click="close"
				>
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>

			<!-- Replace row -->
			<div v-if="isReplaceOpen" class="flex items-center gap-1">
				<div class="w-9 flex-shrink-0" />
				<div @keydown.enter.prevent.stop="emit('replace', replaceQuery)">
					<StyledInput
						ref="replaceInputRef"
						v-model="replaceQuery"
						type="search"
						size="small"
						autocomplete="off"
						:disabled="props.readonly"
						:placeholder="formatMessage(messages.replaceInFile)"
						wrapper-class="w-44"
					/>
				</div>
				<Button type="outlined"
						size="sm"
						class="whitespace-nowrap"
						:disabled="props.readonly || findMatchCount === 0"
						@click="emit('replace', replaceQuery)"
					>
						{{ formatMessage(messages.replace) }}
					</Button>
				<Button type="outlined"
						size="sm"
						class="whitespace-nowrap"
						:disabled="props.readonly || findMatchCount === 0"
						@click="emit('replaceAll', replaceQuery)"
					>
						{{ formatMessage(messages.replaceAll) }}
					</Button>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronRightIcon, ChevronUpIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref, watch } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import IconButton from '#ui/components/base/buttons/IconButton.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	isFindOpen: boolean
	findQuery: string
	findMatchCount: number
	currentFindMatch: number
	isEditingImage: boolean
	readonly?: boolean
}>()

const emit = defineEmits<{
	'update:isFindOpen': [value: boolean]
	'update:findQuery': [value: string]
	close: []
	findNext: []
	findPrevious: []
	replace: [query: string]
	replaceAll: [query: string]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	findInFile: {
		id: 'files.editor.find-in-file',
		defaultMessage: 'Find',
	},
	matchCount: {
		id: 'files.editor.find-match-count',
		defaultMessage: '{current} of {total}',
	},
	noResults: {
		id: 'files.editor.find-no-results',
		defaultMessage: 'No results',
	},
	previousMatch: {
		id: 'files.editor.find-previous-match',
		defaultMessage: 'Previous match',
	},
	nextMatch: {
		id: 'files.editor.find-next-match',
		defaultMessage: 'Next match',
	},
	closeFind: {
		id: 'files.editor.find-close',
		defaultMessage: 'Close',
	},
	toggleReplace: {
		id: 'files.editor.find-toggle-replace',
		defaultMessage: 'Toggle replace',
	},
	replaceInFile: {
		id: 'files.editor.replace-in-file',
		defaultMessage: 'Replace',
	},
	replace: {
		id: 'files.editor.replace',
		defaultMessage: 'Replace',
	},
	replaceAll: {
		id: 'files.editor.replace-all',
		defaultMessage: 'Replace All',
	},
})

const isReplaceOpen = ref(false)
const replaceQuery = ref('')

const findInputRef = ref<{ focus: () => void } | null>(null)
const replaceInputRef = ref<{ focus: () => void } | null>(null)

function toggleReplace() {
	if (props.readonly) return
	isReplaceOpen.value = !isReplaceOpen.value
	if (isReplaceOpen.value) {
		nextTick(() => replaceInputRef.value?.focus())
	}
}

function focusFindInput() {
	nextTick(() => findInputRef.value?.focus())
}

function openReplace() {
	if (props.readonly) return
	isReplaceOpen.value = true
	nextTick(() => replaceInputRef.value?.focus())
}

function close() {
	isReplaceOpen.value = false
	replaceQuery.value = ''
	emit('close')
}

watch(
	() => props.isFindOpen,
	(isOpen) => {
		if (!isOpen) {
			isReplaceOpen.value = false
			replaceQuery.value = ''
		}
	},
)

defineExpose({
	focusFindInput,
	openReplace,
})
</script>

<style scoped>
.find-enter-active,
.find-leave-active {
	transition:
		opacity 0.15s ease,
		transform 0.15s ease;
}

.find-enter-from,
.find-leave-to {
	opacity: 0;
	transform: translateY(-4px) scale(0.97);
}
</style>
