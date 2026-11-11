<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import { Button, NewModal, Textarea } from '@modrinth/ui'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

const emit = defineEmits<{
	apply: [projectIds: string[]]
}>()

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const textareaRef = useTemplateRef<InstanceType<typeof Textarea>>('textareaRef')
const input = ref('')
const error = ref('')
const projectIds = computed(() => parseProjectIds(input.value))

function parseDelimitedRows(value: string, delimiter: string): string[][] {
	const rows: string[][] = []
	let row: string[] = []
	let field = ''
	let quoted = false

	for (let i = 0; i < value.length; i++) {
		const character = value[i]
		if (character === '"') {
			if (quoted && value[i + 1] === '"') {
				field += '"'
				i++
			} else if (quoted || field.trim() === '') {
				quoted = !quoted
			} else {
				field += character
			}
		} else if (!quoted && (character === delimiter || character === '\n' || character === '\r')) {
			row.push(field)
			field = ''
			if (character !== delimiter) {
				rows.push(row)
				row = []
				if (character === '\r' && value[i + 1] === '\n') i++
			}
		} else {
			field += character
		}
	}
	row.push(field)
	rows.push(row)
	return rows
}

function parseProjectIds(value: string): string[] {
	const normalizedValue = value.trim().replace(/^\[([\s\S]*)\]$/, '$1')
	let ids: unknown[]

	try {
		const parsed: unknown = JSON.parse(value)
		ids = Array.isArray(parsed) ? parsed : [parsed]
		ids = ids.map((entry) => {
			if (entry && typeof entry === 'object') {
				return Object.entries(entry).find(([key]) => key.toLowerCase() === 'id')?.[1]
			}
			return entry
		})
	} catch {
		const header = value.trim().split(/[\r\n]/, 1)[0]
		const delimiter = header.includes('\t') ? '\t' : ','
		const rows = parseDelimitedRows(value.trim(), delimiter)
		const idColumn = rows[0].findIndex((column) => column.trim().toLowerCase() === 'id')
		ids =
			idColumn === -1
				? normalizedValue.split(/[,\r\n]+/)
				: rows.slice(1).map((row) => row[idColumn])
	}

	return [
		...new Set(
			ids
				.filter((id): id is string => typeof id === 'string')
				.map((id) => {
					const trimmed = id
						.trim()
						.replace(/^(['"])(.*)\1$/, '$2')
						.replace(/\s+/g, '')
					try {
						return new URL(trimmed).pathname.split('/').filter(Boolean).pop() ?? ''
					} catch {
						return trimmed
					}
				})
				.filter(Boolean),
		),
	]
}

async function show() {
	input.value = ''
	error.value = ''
	modalRef.value?.show()
	await nextTick()
	textareaRef.value?.focus()
}

function hide() {
	modalRef.value?.hide()
}

function apply() {
	if (projectIds.value.length === 0) {
		error.value = 'Enter at least one project ID.'
		return
	}

	emit('apply', projectIds.value)
	hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal ref="modalRef" header="Moderate by IDs" width="36rem" max-width="calc(100vw - 2rem)">
		<form class="flex flex-col gap-4" @submit.prevent="apply">
			<div class="flex flex-col gap-2">
				<label class="font-semibold text-contrast" for="moderation-project-ids">
					Project IDs to moderate
				</label>
				<Textarea
					id="moderation-project-ids"
					ref="textareaRef"
					v-model="input"
					:rows="10"
					:error="!!error"
					resize="vertical"
					placeholder="Enter IDs or links, or paste JSON, CSV, or TSV"
					wrapper-class="min-h-48"
					@input="error = ''"
				/>
				<span v-if="error" class="text-sm font-semibold text-red">{{ error }}</span>
				<span v-else class="text-sm text-secondary">
					Accepts comma- or newline-separated IDs, arrays, or JSON, CSV, and TSV with an ID column.
					Links use the ID at the end of the path.
				</span>
				<p class="m-0 break-words text-sm text-secondary">
					Queue IDs ({{ projectIds.length }}): {{ projectIds.join(', ') }}
				</p>
			</div>

			<div class="flex justify-end gap-2">
				<Button native-type="button" @click="hide">
					<XIcon aria-hidden="true" />
					Cancel
				</Button>
				<Button type="colored" color="brand" native-type="submit">
					<CheckIcon aria-hidden="true" />
					Apply
				</Button>
			</div>
		</form>
	</NewModal>
</template>
