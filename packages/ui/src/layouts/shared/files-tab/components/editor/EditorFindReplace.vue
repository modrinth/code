<template>
	<Transition name="find">
		<div
			v-if="isFindOpen && !isEditingImage"
			class="absolute right-3 top-3 z-10 flex flex-col gap-1 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-1.5 shadow-lg"
			@keydown.escape.stop="close"
		>
			<!-- Find row -->
			<div class="flex items-center gap-1">
				<ButtonStyled type="transparent" circular>
					<button
						v-tooltip="formatMessage(messages.toggleReplace)"
						:aria-label="formatMessage(messages.toggleReplace)"
						@click="toggleReplace"
					>
						<ChevronRightIcon
							class="transition-transform duration-150"
							:class="{ 'rotate-90': isReplaceOpen }"
						/>
					</button>
				</ButtonStyled>
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
				<ButtonStyled type="transparent" circular>
					<button
						v-tooltip="formatMessage(messages.previousMatch)"
						:disabled="findMatchCount === 0"
						:aria-label="formatMessage(messages.previousMatch)"
						@click="emit('findPrevious')"
					>
						<ChevronUpIcon />
					</button>
				</ButtonStyled>
				<ButtonStyled type="transparent" circular>
					<button
						v-tooltip="formatMessage(messages.nextMatch)"
						:disabled="findMatchCount === 0"
						:aria-label="formatMessage(messages.nextMatch)"
						@click="emit('findNext')"
					>
						<ChevronDownIcon />
					</button>
				</ButtonStyled>
				<div class="mx-0.5 h-4 w-px bg-surface-5" />
				<ButtonStyled type="transparent" circular>
					<button
						v-tooltip="formatMessage(messages.closeFind)"
						:aria-label="formatMessage(messages.closeFind)"
						@click="close"
					>
						<XIcon />
					</button>
				</ButtonStyled>
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
						:placeholder="formatMessage(messages.replaceInFile)"
						wrapper-class="w-44"
					/>
				</div>
				<ButtonStyled type="outlined">
					<button
						class="!h-8 whitespace-nowrap !border !border-surface-5 px-2 text-sm disabled:opacity-50"
						:disabled="findMatchCount === 0"
						@click="emit('replace', replaceQuery)"
					>
						{{ formatMessage(messages.replace) }}
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button
						class="!h-8 whitespace-nowrap !border !border-surface-5 px-2 text-sm disabled:opacity-50"
						:disabled="findMatchCount === 0"
						@click="emit('replaceAll', replaceQuery)"
					>
						{{ formatMessage(messages.replaceAll) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronRightIcon, ChevronUpIcon, XIcon } from '@modrinth/assets'
import { nextTick, ref, watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	isFindOpen: boolean
	findQuery: string
	findMatchCount: number
	currentFindMatch: number
	isEditingImage: boolean
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
	isReplaceOpen.value = !isReplaceOpen.value
	if (isReplaceOpen.value) {
		nextTick(() => replaceInputRef.value?.focus())
	}
}

function focusFindInput() {
	nextTick(() => findInputRef.value?.focus())
}

function openReplace() {
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
