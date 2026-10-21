<script setup lang="ts">
import { DownloadIcon, FileIcon } from '@modrinth/assets'
import { Button, defineMessages, NewModal, Textarea, useVIntl } from '@modrinth/ui'
import { ref, useId } from 'vue'

import { useModerationAppearanceTheme, useModerationSettings } from '~/composables/moderation'
import { applyAppearance, exportAppearance, parseAppearance } from '~/helpers/moderation-appearance'

const settings = useModerationSettings()
const theme = useModerationAppearanceTheme()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	import: {
		id: 'moderation.appearance.import',
		defaultMessage: 'Import theme',
	},
	export: {
		id: 'moderation.appearance.export',
		defaultMessage: 'Export theme',
	},
	json: { id: 'moderation.appearance.json', defaultMessage: 'Theme JSON' },
	importHelp: {
		id: 'moderation.appearance.import-help',
		defaultMessage:
			'Paste theme JSON or choose a JSON file, then apply it to preview your appearance.',
	},
	exportHelp: {
		id: 'moderation.appearance.export-help',
		defaultMessage: 'Copy your appearance as JSON or download it to share or import later.',
	},
	file: {
		id: 'moderation.appearance.file',
		defaultMessage: 'Choose JSON file',
	},
	download: {
		id: 'moderation.appearance.download',
		defaultMessage: 'Download JSON',
	},
	copy: { id: 'moderation.appearance.copy', defaultMessage: 'Copy JSON' },
	copied: { id: 'moderation.appearance.copied', defaultMessage: 'Copied!' },
	apply: { id: 'moderation.appearance.apply', defaultMessage: 'Apply theme' },
	invalid: {
		id: 'moderation.appearance.invalid',
		defaultMessage:
			'Invalid theme. Use a version 1 review appearance export with all seven values, valid hex colors, opacity from 0–100, and border width from 0–8.',
	},
	fileError: {
		id: 'moderation.appearance.file-error',
		defaultMessage: 'Could not read this file. Choose a JSON file smaller than 64 KB.',
	},
	copyError: {
		id: 'moderation.appearance.copy-error',
		defaultMessage: 'Could not copy automatically. Select and copy the JSON below.',
	},
})
const root = ref<HTMLElement>()
const fileInput = ref<HTMLInputElement>()
const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'import' | 'export'>('import')
const json = ref('')
const error = ref('')
const copied = ref(false)
const jsonId = useId()
function show(nextMode: 'import' | 'export') {
	mode.value = nextMode
	json.value = nextMode === 'export' ? exportAppearance(settings.value) : ''
	error.value = ''
	copied.value = false
	modal.value?.show()
}

function apply() {
	try {
		const values = parseAppearance(json.value)
		theme.value = { id: 'custom', values }
		applyAppearance(settings.value, values)
		modal.value?.hide()
	} catch {
		error.value = formatMessage(messages.invalid)
	}
}

async function readFile(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	input.value = ''
	if (!file) return
	error.value = ''
	try {
		if (file.size > 65536) throw new Error('size')
		json.value = await file.text()
	} catch {
		error.value = formatMessage(messages.fileError)
	}
}

async function copy() {
	try {
		const view = root.value?.ownerDocument.defaultView
		if (!view) return
		await view.navigator.clipboard.writeText(json.value)
		copied.value = true
	} catch {
		error.value = formatMessage(messages.copyError)
	}
}

function download() {
	const document = root.value?.ownerDocument
	const view = document?.defaultView
	if (!document || !view) return
	const url = view.URL.createObjectURL(new Blob([json.value], { type: 'application/json' }))
	const link = document.createElement('a')
	link.href = url
	link.download = 'modrinth-review-theme.json'
	link.click()
	view.setTimeout(() => view.URL.revokeObjectURL(url), 1000)
}
</script>

<template>
	<div ref="root" class="ml-auto shrink-0">
		<div class="flex flex-wrap gap-2">
			<Button size="sm" @click="show('import')">{{ formatMessage(messages.import) }}</Button>
			<Button size="sm" @click="show('export')">{{ formatMessage(messages.export) }}</Button>
		</div>
		<NewModal
			ref="modal"
			:header="formatMessage(messages[mode])"
			width="40rem"
			max-width="calc(100vw - 2rem)"
		>
			<div class="flex flex-col gap-4">
				<p class="m-0 text-secondary">
					{{ formatMessage(mode === 'import' ? messages.importHelp : messages.exportHelp) }}
				</p>
				<label :for="jsonId" class="font-semibold text-contrast">{{
					formatMessage(messages.json)
				}}</label>
				<Textarea
					:id="jsonId"
					v-model="json"
					:readonly="mode === 'export'"
					:rows="14"
					:spellcheck="false"
					resize="vertical"
					input-class="font-mono text-sm"
					:error="!!error"
					@input="error = ''"
				/>
				<p v-if="error" role="alert" class="m-0 text-sm text-red">
					{{ error }}
				</p>
				<div class="flex flex-wrap justify-end gap-2">
					<template v-if="mode === 'import'">
						<input
							ref="fileInput"
							type="file"
							accept=".json,application/json"
							class="hidden"
							:aria-label="formatMessage(messages.file)"
							@change="readFile"
						/>
						<Button @click="fileInput?.click()">
							<FileIcon aria-hidden="true" />
							{{ formatMessage(messages.file) }}
						</Button>
						<Button type="colored" color="brand" :disabled="!json.trim()" @click="apply">{{
							formatMessage(messages.apply)
						}}</Button>
					</template>
					<template v-else>
						<Button @click="download">
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(messages.download) }}
						</Button>
						<Button type="colored" color="brand" @click="copy">{{
							formatMessage(copied ? messages.copied : messages.copy)
						}}</Button>
					</template>
				</div>
			</div>
		</NewModal>
	</div>
</template>
