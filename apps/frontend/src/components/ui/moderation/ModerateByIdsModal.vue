<script setup lang="ts">
import { CheckIcon, RightArrowIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { Button, Checkbox, Combobox, NewModal, Textarea } from '@modrinth/ui'
import Accordion from '@modrinth/ui/src/components/base/Accordion.vue'
import ButtonFrame from '@modrinth/ui/src/components/base/buttons/ButtonFrame.vue'
import FileInput from '@modrinth/ui/src/components/base/FileInput.vue'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

const props = defineProps<{
	skipProcessedIds?: boolean
}>()

const emit = defineEmits<{
	apply: [projectIds: string[], skipProcessedIds: boolean]
}>()

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const textareaRef = useTemplateRef<InstanceType<typeof Textarea>>('textareaRef')
const input = ref('')
const error = ref('')
const skipProcessedIds = ref(props.skipProcessedIds ?? true)
const selectedColumn = ref('')
const output = computed<(string | null)[] | { [column: string]: (string | null)[] }>(() =>
	parseProjectIds(input.value),
)
const projectIds = computed<string[]>(() => {
	let set: Set<string | null> = new Set()
	if (Array.isArray(output.value)) {
		set = new Set(output.value)
	} else if (typeof output.value === 'object' && output.value[selectedColumn.value]) {
		set = new Set(output.value[selectedColumn.value])
	} else if (typeof output.value === 'object') {
		set = new Set(Object.keys(output.value).concat(...Object.values(output.value)))
	}
	return [...set].filter(Boolean)
})

function parseProjectIds(
	input: string,
): (string | null)[] | { [column: string]: (string | null)[] } {
	try {
		const parsed = JSON.parse(input)
		if (Array.isArray(parsed) && parsed.length) {
			if (
				!parsed.every(
					(item) =>
						typeof item === typeof parsed[0] &&
						!Array.isArray(item) &&
						(typeof item === 'string' || typeof item === 'object'),
				)
			) {
				error.value = 'All items in the array must be of the same type, strings or objects.'
				return []
			} else if (typeof parsed[0] === 'string') {
				return parsed.map(normalizeId)
			} else {
				const output = {} as any
				let index = 0

				for (const item of parsed) {
					for (const key in item) {
						if (!output[key]) {
							output[key] = []
							output[key].fill(null, 0, index)
						}
						output[key].push(normalizeId(item[key]))
					}
					index++
				}

				return output
			}
		}
	} catch {
		// Ignore JSON errors fallback to parsing as text as other formats
	}

	input = input.trim()
	input =
		input.startsWith('[') && input.endsWith(']') ? input.substring(1, input.length - 1) : input

	const lines = input.split(/\r?\n/).filter(Boolean)
	if (lines.length) {
		if (lines[0].includes(',') || lines[0].includes('\t')) {
			const delimiter = lines[0].includes('\t') ? '\t' : ','
			const headers = lines[0]
				.split(delimiter)
				.map((header) => header.trim())
				.filter(Boolean)

			if (headers.length < 2 || lines.length < 2) {
				return lines.map((line) => line.split(/[,\t]/).map(normalizeId)).flat()
			}

			const output = {} as any
			for (const header of headers) {
				output[header] = []
			}
			for (let i = 1; i < lines.length; i++) {
				const values = lines[i].split(delimiter).map((value) => value.trim())
				for (let j = 0; j < headers.length; j++) {
					output[headers[j]].push(normalizeId(values[j] || ''))
				}
			}
			return output
		} else {
			return lines.map(normalizeId)
		}
	}

	return []
}

function normalizeId(item: string): string {
	const url = URL.parse(item)
	if (url && url.searchParams.has('project')) {
		return url.searchParams.get('project').trim()
	} else if (url && url.pathname) {
		return url.pathname.split('/').filter(Boolean).pop().trim()
	}
	return item.trim()
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
	if (output.value.length === 0) {
		error.value = 'Enter at least one project ID.'
		return
	}

	emit('apply', projectIds.value, skipProcessedIds.value)
	hide()
}

async function onFileChange(files: File[]) {
	const file = files[0]
	if (!file) {
		return
	}
	const text = await file.text()
	if (text && !text.includes('\0')) {
		input.value = text
	}
}

defineExpose({ show, hide })

watch(output, (parsed, _) => {
	if (typeof parsed === 'object' && !Array.isArray(parsed)) {
		if (!selectedColumn.value || !parsed[selectedColumn.value]) {
			selectedColumn.value = Object.keys(parsed)[0]
		}
	} else {
		selectedColumn.value = ''
	}
})
</script>

<template>
	<NewModal ref="modalRef" header="Moderate by IDs" width="36rem" max-width="calc(100vw - 2rem)">
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<div class="flex items-center justify-between">
						<div class="flex flex-col content-start gap-1">
							<span class="text-left font-semibold text-contrast">Input IDs</span>
							<span class="text-sm text-secondary">
								Enter IDs or upload a file containing IDs.
							</span>
						</div>
						<Button size="sm">
							<FileInput
								accept="text/csv,text/plain,.csv,.tsv,.json"
								:prompt="'Upload File'"
								should-always-reset
								clear-on-change
								class="flex cursor-pointer items-center gap-1"
								@change="onFileChange"
							>
								<UploadIcon />
							</FileInput>
						</Button>
					</div>

					<Textarea
						id="moderation-project-ids"
						ref="textareaRef"
						v-model="input"
						:rows="3"
						:error="!!error"
						resize="vertical"
						placeholder="Enter IDs or links, or paste JSON, CSV, or TSV"
						wrapper-class="min-h-48 flex flex-col !px-0"
						input-class="px-3 border-solid border-b border-surface-5"
						@input="error = ''"
					>
						<template #right>
							<div class="flex items-center gap-2 px-3">
								<Checkbox id="moderation-skip-processed-ids" v-model="skipProcessedIds" />
								<label class="text-xs text-secondary" for="moderation-skip-processed-ids">
									Skip IDs that have already been processed
								</label>
							</div>
						</template>
					</Textarea>

					<span v-if="error" class="text-sm font-semibold text-red">{{ error }}</span>
				</div>

				<div
					v-if="!Array.isArray(output) && typeof output === 'object'"
					class="flex flex-col gap-2"
				>
					<div class="flex flex-col content-start gap-1">
						<span class="text-left font-semibold text-contrast">Column Mapping</span>
						<span class="text-sm text-secondary">Select columns to use for IDs.</span>
					</div>

					<div
						class="grid grid-cols-[1fr_auto_1fr] items-center gap-2 rounded-xl border border-solid border-surface-5 bg-surface-4 px-3 py-3"
					>
						<ButtonFrame as="test" interaction="none" disabled="true" aria-disabled="true">
							<span class="w-full text-left">id</span>
						</ButtonFrame>
						<RightArrowIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
						<Combobox
							v-model="selectedColumn"
							trigger-type="outlined"
							:options="[
								{ label: 'Unset', value: '' },
								...Object.keys(output).map((item) => ({ label: item, value: item })),
							]"
						/>
					</div>
				</div>

				<Accordion button-class="p-0 w-full py-4">
					<template #title>
						<div class="flex flex-col content-start gap-1">
							<span class="text-left font-semibold text-contrast"
								>Preview ({{ projectIds.length }} IDs)</span
							>
							<span class="text-sm text-secondary">Preview the IDs that will be moderated.</span>
						</div>
					</template>

					<div class="rounded-xl border border-solid border-surface-5 bg-surface-4 pb-3">
						<div
							class="flex flex-row gap-4 rounded-t-xl border-0 border-b border-solid border-surface-5 bg-surface-4 px-3 py-2"
						>
							<span class="text-sm font-semibold text-contrast">#</span>
							<span class="text-sm font-semibold text-contrast">ID</span>
						</div>
						<div class="flex max-h-32 flex-col overflow-y-auto">
							<div v-if="projectIds.length === 0" class="flex items-center justify-center py-4">
								<span class="text-sm text-secondary">No IDs to preview</span>
							</div>
							<div
								v-for="(id, index) in projectIds"
								:key="id"
								class="flex flex-row gap-4 border-0 border-b border-solid border-surface-5 bg-surface-4 px-3 py-1"
							>
								<span class="text-sm text-secondary">{{ index + 1 }}</span>
								<span class="break-all text-sm text-contrast">{{ id }}</span>
							</div>
						</div>
					</div>
				</Accordion>
			</div>

			<div class="flex justify-end gap-2">
				<Button @click="hide">
					<XIcon aria-hidden="true" />
					Cancel
				</Button>
				<Button type="colored" color="brand" @click="apply">
					<CheckIcon aria-hidden="true" />
					Apply
				</Button>
			</div>
		</div>
	</NewModal>
</template>
