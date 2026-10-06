<script setup lang="ts">
import { CircleAlertIcon, FolderSearchIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	Input,
	LargeRadioButton,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useId } from 'vue'

const props = defineProps<{ instanceSettings?: boolean }>()
const { formatMessage } = useVIntl()
const id = useId()
const customSettings = ref(false)
const disabled = computed(() => props.instanceSettings && !customSettings.value)
const mode = ref('unrecognized')
const permissions = ref({ network: true, downloads: true, documents: false })
const folderPath = ref('')
const folders = ref<string[]>([])
const canAddFolder = computed(() => {
	const path = folderPath.value.trim()
	return !disabled.value && path.length > 0 && !folders.value.includes(path)
})

const messages = defineMessages({
	customDescription: {
		id: 'app.settings.sandbox.custom-description',
		defaultMessage: 'Override app sandbox settings for this instance.',
	},
	custom: {
		id: 'app.settings.sandbox.custom',
		defaultMessage: 'Custom sandbox settings',
	},
	title: {
		id: 'app.settings.sandbox.title',
		defaultMessage: 'Sandbox instances',
	},
	instanceTitle: {
		id: 'app.settings.sandbox.instance.title',
		defaultMessage: 'Sandbox instance',
	},
	description: {
		id: 'app.settings.sandbox.description',
		defaultMessage:
			'Runs Minecraft in an isolated environment, so mods can only access your .minecraft folder and anything you allow below.',
	},
	never: {
		id: 'app.settings.sandbox.never',
		defaultMessage: 'Never',
	},
	neverDescription: {
		id: 'app.settings.sandbox.never.description',
		defaultMessage: 'Run instances with full access to your computer.',
	},
	instanceNeverDescription: {
		id: 'app.settings.sandbox.instance.never.description',
		defaultMessage: 'Run this instance with full access to your computer.',
	},
	unrecognized: {
		id: 'app.settings.sandbox.unrecognized',
		defaultMessage: 'Unrecognized files',
	},
	unrecognizedDescription: {
		id: 'app.settings.sandbox.unrecognized.description',
		defaultMessage: 'Sandbox instances containing files not from Modrinth.',
	},
	instanceUnrecognizedDescription: {
		id: 'app.settings.sandbox.instance.unrecognized.description',
		defaultMessage: 'Sandbox this instance if it contains files not from Modrinth.',
	},
	always: {
		id: 'app.settings.sandbox.always',
		defaultMessage: 'Always',
	},
	alwaysDescription: {
		id: 'app.settings.sandbox.always.description',
		defaultMessage: 'Run instances always in the sandbox.',
	},
	instanceAlwaysDescription: {
		id: 'app.settings.sandbox.instance.always.description',
		defaultMessage: 'Always run this instance in the sandbox.',
	},
	warning: {
		id: 'app.settings.sandbox.warning',
		defaultMessage: 'Some mods may not work if they require access blocked by the sandbox.',
	},
	network: {
		id: 'app.settings.sandbox.network',
		defaultMessage: 'Access network',
	},
	networkDescription: {
		id: 'app.settings.sandbox.network.description',
		defaultMessage: 'Allows connections to the internet, multiplayer servers, and online services.',
	},
	downloads: {
		id: 'app.settings.sandbox.downloads',
		defaultMessage: 'Access downloads folder',
	},
	downloadsDescription: {
		id: 'app.settings.sandbox.downloads.description',
		defaultMessage: 'Allows access to files in your Downloads folder.',
	},
	documents: {
		id: 'app.settings.sandbox.documents',
		defaultMessage: 'Access documents folder',
	},
	documentsDescription: {
		id: 'app.settings.sandbox.documents.description',
		defaultMessage: 'Allows access to files in your Documents folder.',
	},
	folders: {
		id: 'app.settings.sandbox.folders',
		defaultMessage: 'Access other folders',
	},
	foldersDescription: {
		id: 'app.settings.sandbox.folders.description',
		defaultMessage:
			'Allows mods to read or modify files you choose. Only add folders you’re comfortable giving them access to.',
	},
	folderPlaceholder: {
		id: 'app.settings.sandbox.folders.placeholder',
		defaultMessage: 'Path to folder...',
	},
	browse: {
		id: 'app.settings.sandbox.folders.browse',
		defaultMessage: 'Browse for a folder',
	},
	add: {
		id: 'app.settings.sandbox.folders.add',
		defaultMessage: 'Add',
	},
})

const modes = [
	{
		value: 'never',
		label: messages.never,
		description: messages.neverDescription,
		instanceDescription: messages.instanceNeverDescription,
	},
	{
		value: 'unrecognized',
		label: messages.unrecognized,
		description: messages.unrecognizedDescription,
		instanceDescription: messages.instanceUnrecognizedDescription,
	},
	{
		value: 'always',
		label: messages.always,
		description: messages.alwaysDescription,
		instanceDescription: messages.instanceAlwaysDescription,
	},
]
const permissionOptions = [
	{ key: 'network', label: messages.network, description: messages.networkDescription },
	{ key: 'downloads', label: messages.downloads, description: messages.downloadsDescription },
	{ key: 'documents', label: messages.documents, description: messages.documentsDescription },
] as const

function addFolder() {
	if (!canAddFolder.value) return
	folders.value.push(folderPath.value.trim())
	folderPath.value = ''
}

function selectAdjacentMode(event: KeyboardEvent, index: number) {
	const directions: Record<string, number> = { ArrowLeft: -1, ArrowUp: -1, ArrowRight: 1, ArrowDown: 1 }
	const direction = directions[event.key]
	if (direction === undefined || disabled.value) return
	event.preventDefault()
	const nextIndex = (index + direction + modes.length) % modes.length
	mode.value = modes[nextIndex].value
	const button = event.currentTarget as HTMLElement
	button.parentElement?.querySelectorAll<HTMLButtonElement>('[role="radio"]')[nextIndex]?.focus()
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div
			v-if="instanceSettings"
			class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 pb-6"
		>
			<div class="flex min-w-0 flex-col gap-0.5">
				<label :for="`${id}-custom`" class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.custom) }}
				</label>
				<p :id="`${id}-custom-description`" class="m-0">
					{{ formatMessage(messages.customDescription) }}
				</p>
			</div>
			<Toggle
				:id="`${id}-custom`"
				v-model="customSettings"
				:aria-describedby="`${id}-custom-description`"
			/>
		</div>
		<fieldset
			:disabled="disabled"
			class="m-0 min-w-0 border-0 p-0"
			:class="{ 'opacity-50': disabled }"
		>
			<div class="flex flex-col gap-6">
				<div class="flex flex-col gap-4">
					<div class="flex flex-col gap-0.5">
						<h2 :id="`${id}-title`" class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(instanceSettings ? messages.instanceTitle : messages.title) }}
						</h2>
						<p class="m-0 leading-6">{{ formatMessage(messages.description) }}</p>
					</div>
					<div role="radiogroup" :aria-labelledby="`${id}-title`" class="grid grid-cols-3 gap-3">
						<LargeRadioButton
							v-for="(option, index) in modes"
							:key="option.value"
							type="button"
							:selected="mode === option.value"
							:tabindex="mode === option.value ? 0 : -1"
							class="!grid min-w-0 grid-cols-[20px_1fr] content-start items-center !gap-x-2 !gap-y-1.5 !rounded-[20px] !border !p-3 !font-normal !leading-6"
							:class="
								mode === option.value
									? '!border-brand !bg-brand-highlight'
									: '!border-surface-4 !bg-surface-3'
							"
							@select="mode = option.value"
							@keydown="selectAdjacentMode($event, index)"
						>
							<span class="font-semibold text-contrast">{{ formatMessage(option.label) }}</span>
							<span class="col-span-2">{{
								formatMessage(instanceSettings ? option.instanceDescription : option.description)
							}}</span>
						</LargeRadioButton>
					</div>
					<div
						class="flex items-start gap-2 rounded-2xl border border-solid border-orange bg-highlight-orange p-4 text-contrast"
					>
						<CircleAlertIcon class="size-6 shrink-0 text-orange" aria-hidden="true" />
						<p class="m-0 leading-6">{{ formatMessage(messages.warning) }}</p>
					</div>
				</div>
				<div class="flex flex-col gap-3 border-0 border-t border-solid border-surface-5 pt-6">
					<div
						v-for="option in permissionOptions"
						:key="option.key"
						class="flex items-center justify-between gap-4"
					>
						<div class="flex flex-col gap-0.5">
							<label :for="`${id}-${option.key}`" class="text-lg font-semibold text-contrast">
								{{ formatMessage(option.label) }}
							</label>
							<p :id="`${id}-${option.key}-description`" class="m-0 leading-6">
								{{ formatMessage(option.description) }}
							</p>
						</div>
						<Toggle
							:id="`${id}-${option.key}`"
							v-model="permissions[option.key]"
							:aria-describedby="`${id}-${option.key}-description`"
						/>
					</div>
				</div>
				<div class="flex flex-col gap-2.5 border-0 border-t border-solid border-surface-5 pt-6">
					<div class="flex flex-col gap-0.5">
						<h2 class="m-0 text-lg font-semibold text-contrast">
							{{ formatMessage(messages.folders) }}
						</h2>
						<p class="m-0 leading-6">{{ formatMessage(messages.foldersDescription) }}</p>
					</div>
					<form class="flex gap-2" @submit.prevent="addFolder">
						<Input
							v-model="folderPath"
							class="min-w-0 flex-1"
							size="medium"
							:aria-label="formatMessage(messages.folderPlaceholder)"
							:placeholder="formatMessage(messages.folderPlaceholder)"
						/>
						<IconButton :label="formatMessage(messages.browse)" size="lg" disabled>
							<FolderSearchIcon />
						</IconButton>
						<Button
							native-type="submit"
							type="colored"
							color="brand"
							size="lg"
							:disabled="!canAddFolder"
						>
							<PlusIcon /> {{ formatMessage(messages.add) }}
						</Button>
					</form>
					<div v-for="(folder, index) in folders" :key="folder" class="flex items-center gap-2">
						<Input
							:model-value="folder"
							readonly
							class="min-w-0 flex-1"
							size="medium"
							:aria-label="formatMessage(messages.folderPlaceholder)"
						/>
						<IconButton
							:label="formatMessage(commonMessages.removeButton)"
							type="quiet"
							size="lg"
							@click="folders.splice(index, 1)"
						>
							<XIcon />
						</IconButton>
					</div>
				</div>
			</div>
		</fieldset>
	</div>
</template>
