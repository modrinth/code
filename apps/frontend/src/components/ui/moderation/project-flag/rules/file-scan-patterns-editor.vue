<template>
	<section ref="section" class="flex min-w-0 flex-col gap-3" :aria-labelledby="headingId">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 :id="headingId" class="m-0 text-base font-semibold text-contrast">File scan patterns</h3>
			<Button :disabled="disabled" @click="addPattern">
				<PlusIcon />
				Add pattern
			</Button>
		</div>
		<p v-if="patterns.length === 0" class="m-0 text-sm text-secondary">No file scan patterns.</p>
		<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
			<li
				v-for="(entry, index) in patterns"
				:key="rowId(entry)"
				class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-start gap-2 rounded-xl border border-solid border-surface-4 p-3 md:grid-cols-[minmax(0,1fr)_minmax(0,2fr)_auto]"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<label :for="`${rowId(entry)}-name`" class="text-sm font-medium text-secondary"
						>Name</label
					>
					<Input
						:id="`${rowId(entry)}-name`"
						v-model="entry.name"
						:disabled="disabled"
						:error="Boolean(errors[index]?.name)"
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
				<div
					class="col-start-1 row-start-2 flex min-w-0 flex-col gap-1 md:col-start-2 md:row-start-1"
				>
					<label :for="`${rowId(entry)}-regex`" class="text-sm font-medium text-secondary"
						>Regex</label
					>
					<Input
						:id="`${rowId(entry)}-regex`"
						v-model="entry.pattern"
						:disabled="disabled"
						:error="Boolean(errors[index]?.pattern)"
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
				<IconButton
					class="col-start-2 row-start-1 mt-6 md:col-start-3"
					type="quiet"
					color="red"
					:disabled="disabled"
					:circular="false"
					:label="`Remove pattern ${index + 1}`"
					@click="patterns.splice(index, 1)"
				>
					<TrashIcon />
				</IconButton>
			</li>
		</ul>
	</section>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon } from '@modrinth/assets'
import { Button, IconButton, Input } from '@modrinth/ui'
import { nextTick, useId, useTemplateRef } from 'vue'

export interface FileScanPattern {
	name: string
	pattern: string
}

const patterns = defineModel<FileScanPattern[]>({ required: true })
defineProps<{ disabled?: boolean; errors: FileScanPattern[] }>()

const id = useId()
const headingId = `${id}-heading`
const section = useTemplateRef<HTMLElement>('section')
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

async function addPattern() {
	patterns.value.push({ name: '', pattern: '' })
	const nameId = `${rowId(patterns.value[patterns.value.length - 1])}-name`
	await nextTick()
	section.value?.querySelector<HTMLInputElement>(`[id="${nameId}"]`)?.focus()
}
</script>
