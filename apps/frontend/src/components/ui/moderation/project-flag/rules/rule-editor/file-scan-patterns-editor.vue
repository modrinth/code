<template>
	<section ref="section" class="flex min-w-0 flex-col gap-1" :aria-labelledby="headingId">
		<div class="flex flex-wrap items-center justify-between gap-2 lg:min-h-[38px]">
			<h3 :id="headingId" class="m-0 text-sm font-semibold text-primary">Patterns</h3>
			<Button
				type="quiet"
				size="sm"
				aria-label="Add pattern"
				:disabled="disabled"
				@click="addPattern"
			>
				<PlusIcon />
				Add
			</Button>
		</div>
		<p v-if="!patterns.length" class="m-0 text-sm text-secondary">No patterns.</p>
		<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
			<li
				v-for="(entry, index) in patterns"
				:key="rowId(entry)"
				class="overflow-hidden rounded-xl border border-solid border-surface-4 bg-surface-3"
			>
				<Accordion
					:open-by-default="isExpanded(entry, index)"
					button-class="flex w-full cursor-pointer items-center gap-2 border-0 bg-surface-3 px-3 py-2 text-left text-sm font-medium text-contrast transition-[filter] hover:brightness-125 focus-visible:-outline-offset-2 focus-visible:brightness-125 motion-reduce:transition-none"
					content-class="flex min-w-0 flex-col gap-3 px-3 pb-3 pt-1"
					@on-open="expandedId = rowId(entry)"
					@on-close="expandedId = null"
				>
					<template #button="{ open }">
						<span class="min-w-0 flex-1 truncate">{{ entry.name.trim() || 'New pattern' }}</span>
						<ChevronDownIcon
							class="size-4 shrink-0 text-secondary transition-transform duration-300 motion-reduce:transition-none"
							:class="{ 'rotate-180': open }"
							aria-hidden="true"
						/>
					</template>
					<div class="flex min-w-0 flex-col gap-1">
						<label :for="`${rowId(entry)}-name`" class="text-sm font-medium text-secondary"
							>Name</label
						>
						<Input
							:id="`${rowId(entry)}-name`"
							v-model="entry.name"
							:disabled="disabled"
							:aria-describedby="errors[index]?.name ? `${rowId(entry)}-name-error` : undefined"
							placeholder="pattern_name"
							autocomplete="off"
							@keydown.enter.prevent
						/>
						<p
							v-if="errors[index]?.name"
							:id="`${rowId(entry)}-name-error`"
							class="m-0 text-sm text-red"
						>
							{{ errors[index].name }}
						</p>
					</div>
					<div class="flex min-w-0 flex-col gap-1">
						<label :for="`${rowId(entry)}-regex`" class="text-sm font-medium text-secondary"
							>Regex</label
						>
						<Input
							:id="`${rowId(entry)}-regex`"
							v-model="entry.pattern"
							:disabled="disabled"
							:aria-describedby="errors[index]?.pattern ? `${rowId(entry)}-regex-error` : undefined"
							placeholder="(?i)forbidden text"
							input-class="font-mono"
							autocomplete="off"
							:spellcheck="false"
							@keydown.enter.prevent
						/>
						<p
							v-if="errors[index]?.pattern"
							:id="`${rowId(entry)}-regex-error`"
							class="m-0 text-sm text-red"
						>
							{{ errors[index].pattern }}
						</p>
					</div>
					<div class="flex min-w-0 items-center gap-1">
						<p class="m-0 min-w-0 flex-1 break-words text-xs text-secondary">
							Matched names appear in <code class="text-link">scanned_files[].found_patterns</code>.
						</p>
						<IconButton
							class="shrink-0 enabled:hover:text-red"
							type="quiet"
							size="sm"
							:disabled="disabled"
							:label="`Remove pattern ${index + 1}`"
							@click="removePattern(index)"
						>
							<TrashIcon aria-hidden="true" />
						</IconButton>
					</div>
				</Accordion>
			</li>
		</ul>
	</section>
</template>

<script setup lang="ts">
import { ChevronDownIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import { Accordion, Button, IconButton, Input } from '@modrinth/ui'
import { nextTick, ref, useId, useTemplateRef } from 'vue'

import { useUndoableDraftRemoval } from './use-undoable-draft-removal'

export interface FileScanPattern {
	name: string
	pattern: string
}

const patterns = defineModel<FileScanPattern[]>({ required: true })
const props = defineProps<{ disabled?: boolean; errors: FileScanPattern[] }>()
const removePattern = useUndoableDraftRemoval(
	patterns,
	() => props.disabled,
	({ name }) => ({ title: 'File scan pattern removed', text: name.trim() || undefined }),
)

const id = useId()
const headingId = `${id}-heading`
const section = useTemplateRef<HTMLElement>('section')
const expandedId = ref<string | null>()
const rowIds = new WeakMap<FileScanPattern, string>()
let nextRowId = 0

function rowId(entry: FileScanPattern) {
	let row = rowIds.get(entry)
	if (!row) {
		row = `${id}-pattern-${nextRowId++}`
		rowIds.set(entry, row)
	}
	return row
}

function isExpanded(entry: FileScanPattern, index: number) {
	return expandedId.value === undefined ? index === 0 : expandedId.value === rowId(entry)
}

async function addPattern() {
	patterns.value.push({ name: '', pattern: '' })
	expandedId.value = rowId(patterns.value[patterns.value.length - 1])
	const nameId = `${expandedId.value}-name`
	await nextTick()
	section.value?.querySelector<HTMLInputElement>(`[id="${nameId}"]`)?.focus()
}
</script>
