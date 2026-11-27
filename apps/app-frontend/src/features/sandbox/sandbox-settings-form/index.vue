<script setup lang="ts">
import {
	CheckIcon,
	ClipboardCopyIcon,
	ExternalIcon,
	FolderSearchIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	injectPageContext,
	Input,
	IntlFormatted,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onBeforeUnmount, ref, useId } from 'vue'

import type { SandboxSettings } from '@/helpers/types'

import DisableSandboxModal from './disable-sandbox-modal.vue'
import { SANDBOX_HELP_URL } from './links'

const props = defineProps<{
	settings: SandboxSettings
	instanceSettings?: boolean
	disabled?: boolean
	unavailableReason?: string | null
}>()
const emit = defineEmits<{ update: [patch: Partial<SandboxSettings>] }>()
const { formatMessage } = useVIntl()
const { openExternalUrl } = injectPageContext()
const id = useId()
const folderPath = ref('')
const disableModal = ref<InstanceType<typeof DisableSandboxModal>>()
const sandboxed = computed(() => props.settings.mode !== 'never')
const canAddFolder = computed(() => {
	const path = folderPath.value.trim()
	return path.length > 0 && !props.settings.extra_paths.includes(path)
})

function update<K extends keyof SandboxSettings>(key: K, value: SandboxSettings[K]) {
	emit('update', { [key]: value })
}

const messages = defineMessages({
	info: {
		id: 'app.settings.sandbox.info',
		defaultMessage:
			'The sandbox isolates Minecraft, limiting mods to your system files, .minecraft, and anything we allow or you allow below. Some mods may not work if the sandbox blocks access they need. <link>Learn more</link>',
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
		defaultMessage: 'Launch instances in a sandbox to restrict what mods can access.',
	},
	instanceDescription: {
		id: 'app.settings.sandbox.instance.description',
		defaultMessage: 'Launch this instance in a sandbox to restrict what mods can access.',
	},
	unavailable: {
		id: 'app.settings.sandbox.unavailable',
		defaultMessage:
			'Sandboxing is unavailable on this system, so instances that need the sandbox will not launch: {reason}',
	},
	settingsTitle: {
		id: 'app.settings.sandbox.settings-title',
		defaultMessage: 'Sandbox settings',
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
		defaultMessage: 'Read-only access to downloads folder',
	},
	downloadsDescription: {
		id: 'app.settings.sandbox.downloads.description',
		defaultMessage: 'Allows mods to read files in your Downloads folder.',
	},
	documents: {
		id: 'app.settings.sandbox.documents',
		defaultMessage: 'Read-only access to documents folder',
	},
	documentsDescription: {
		id: 'app.settings.sandbox.documents.description',
		defaultMessage: 'Allows mods to read files in your Documents folder.',
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

const permissionOptions = [
	{
		key: 'allow_network',
		label: messages.network,
		description: messages.networkDescription,
	},
	{
		key: 'allow_downloads',
		label: messages.downloads,
		description: messages.downloadsDescription,
	},
	{
		key: 'allow_documents',
		label: messages.documents,
		description: messages.documentsDescription,
	},
] as const

function setSandboxed(value: boolean) {
	if (value) {
		update('mode', 'always')
	} else {
		disableModal.value?.show()
	}
}

function addFolder() {
	if (!canAddFolder.value) return
	update('extra_paths', [...props.settings.extra_paths, folderPath.value.trim()])
	folderPath.value = ''
}

async function browseForFolder() {
	const path = await open({ multiple: false, directory: true })
	if (typeof path === 'string') folderPath.value = path
}

const copiedFolder = ref<string | null>(null)
let copiedResetTimeout: ReturnType<typeof setTimeout> | undefined

async function copyFolder(folder: string) {
	await navigator.clipboard.writeText(folder)
	copiedFolder.value = folder
	clearTimeout(copiedResetTimeout)
	copiedResetTimeout = setTimeout(() => {
		copiedFolder.value = null
	}, 1500)
}

onBeforeUnmount(() => clearTimeout(copiedResetTimeout))
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-4">
			<Admonition type="info" class="transition-opacity" :class="{ 'opacity-60': disabled }">
				<p class="m-0 leading-6">
					<IntlFormatted :message-id="messages.info">
						<template #link="{ children }">
							<button
								type="button"
								class="inline-flex cursor-pointer items-center gap-1.5 border-0 bg-transparent p-0 align-bottom text-[length:inherit] font-medium leading-6 text-blue hover:underline"
								@click="openExternalUrl(SANDBOX_HELP_URL)"
							>
								<component :is="() => children" />
								<ExternalIcon class="size-4" aria-hidden="true" />
							</button>
						</template>
					</IntlFormatted>
				</p>
			</Admonition>
			<Admonition
				v-if="unavailableReason && sandboxed"
				type="critical"
				:body="formatMessage(messages.unavailable, { reason: unavailableReason })"
			/>
			<div
				class="flex items-center justify-between gap-4 transition-opacity"
				:class="{ 'opacity-60': disabled }"
				:inert="disabled"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<label :for="`${id}-mode`" class="text-lg font-semibold text-contrast">
						{{ formatMessage(instanceSettings ? messages.instanceTitle : messages.title) }}
					</label>
					<p :id="`${id}-mode-description`" class="m-0">
						{{
							formatMessage(instanceSettings ? messages.instanceDescription : messages.description)
						}}
					</p>
				</div>
				<Toggle
					:id="`${id}-mode`"
					class="shrink-0"
					:model-value="sandboxed"
					:aria-describedby="`${id}-mode-description`"
					@update:model-value="setSandboxed"
				/>
			</div>
		</div>
		<div class="border-0 border-t border-solid border-surface-5 pt-6">
			<div
				class="transition-opacity"
				:class="{ 'opacity-60': disabled || !sandboxed }"
				:inert="disabled || !sandboxed"
			>
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.settingsTitle) }}
				</h2>
				<div class="mt-4 flex flex-col gap-6">
					<div
						v-for="option in permissionOptions"
						:key="option.key"
						class="flex items-center justify-between gap-4"
					>
						<div class="flex min-w-0 flex-col gap-1">
							<label :for="`${id}-${option.key}`" class="text-lg font-semibold text-contrast">
								{{ formatMessage(option.label) }}
							</label>
							<p :id="`${id}-${option.key}-description`" class="m-0">
								{{ formatMessage(option.description) }}
							</p>
						</div>
						<Toggle
							:id="`${id}-${option.key}`"
							class="shrink-0"
							:model-value="settings[option.key]"
							:aria-describedby="`${id}-${option.key}-description`"
							@update:model-value="(value) => update(option.key, value)"
						/>
					</div>
					<div class="flex flex-col gap-2.5">
						<div class="flex min-w-0 flex-col gap-1">
							<h3 class="m-0 text-lg font-semibold text-contrast">
								{{ formatMessage(messages.folders) }}
							</h3>
							<p class="m-0">{{ formatMessage(messages.foldersDescription) }}</p>
						</div>
						<form class="flex gap-2" @submit.prevent="addFolder">
							<Input
								v-model="folderPath"
								class="min-w-0 flex-1"
								size="medium"
								:aria-label="formatMessage(messages.folderPlaceholder)"
								:placeholder="formatMessage(messages.folderPlaceholder)"
							/>
							<IconButton
								:label="formatMessage(messages.browse)"
								size="lg"
								@click="browseForFolder"
							>
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
						<div
							v-for="(folder, index) in settings.extra_paths"
							:key="folder"
							class="flex items-center gap-2"
						>
							<Button
								size="lg"
								class="min-w-0 flex-1 !justify-between text-left"
								@click="copyFolder(folder)"
							>
								<span class="min-w-0 truncate text-base font-semibold text-primary">
									{{ folder }}
								</span>
								<CheckIcon
									v-if="copiedFolder === folder"
									class="size-5 shrink-0"
									aria-hidden="true"
								/>
								<ClipboardCopyIcon
									v-else
									class="size-5 shrink-0 text-secondary"
									aria-hidden="true"
								/>
							</Button>
							<IconButton
								:label="formatMessage(commonMessages.removeButton)"
								type="quiet"
								color="red"
								interaction="filled"
								size="lg"
								class="!text-primary"
								@click="
									update(
										'extra_paths',
										settings.extra_paths.filter((_, i) => i !== index),
									)
								"
							>
								<TrashIcon />
							</IconButton>
						</div>
					</div>
				</div>
			</div>
		</div>
		<DisableSandboxModal ref="disableModal" @confirm="update('mode', 'never')" />
	</div>
</template>
