<script setup lang="ts">
import {
	FileIcon,
	FolderSearchIcon,
	LinkIcon,
	PlusIcon,
	SaveIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'

import { set_file_links } from '@/helpers/profile'
import type { FileLink, FileLinkKind } from '@/helpers/types'
import { injectInstanceSettings } from '@/providers/instance-settings'

type FileLinkDraft = FileLink & {
	id: number
}

const { instance } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()

const saving = ref(false)
const draftLinks = ref<FileLinkDraft[]>([])
let nextDraftId = 0

const messages = defineMessages({
	title: {
		id: 'instance.settings.tabs.file-links.title',
		defaultMessage: 'File links',
	},
	description: {
		id: 'instance.settings.tabs.file-links.description',
		defaultMessage:
			'Link files or folders inside this instance to existing locations on your device. Use paths relative to the instance root, like screenshots or options.txt.',
	},
	noLinks: {
		id: 'instance.settings.tabs.file-links.empty',
		defaultMessage: 'No file links configured for this instance yet.',
	},
	pathLabel: {
		id: 'instance.settings.tabs.file-links.path-label',
		defaultMessage: 'Path in instance',
	},
	targetLabel: {
		id: 'instance.settings.tabs.file-links.target-label',
		defaultMessage: 'Target on this device',
	},
	pathPlaceholder: {
		id: 'instance.settings.tabs.file-links.path-placeholder',
		defaultMessage: 'screenshots',
	},
	targetPlaceholder: {
		id: 'instance.settings.tabs.file-links.target-placeholder',
		defaultMessage: 'Select an existing file or folder',
	},
	folderType: {
		id: 'instance.settings.tabs.file-links.folder',
		defaultMessage: 'Folder',
	},
	fileType: {
		id: 'instance.settings.tabs.file-links.file',
		defaultMessage: 'File',
	},
	browseTarget: {
		id: 'instance.settings.tabs.file-links.browse',
		defaultMessage: 'Browse',
	},
	addLink: {
		id: 'instance.settings.tabs.file-links.add-link',
		defaultMessage: 'Add link',
	},
	saveLinks: {
		id: 'instance.settings.tabs.file-links.save',
		defaultMessage: 'Apply file links',
	},
	savingLinks: {
		id: 'instance.settings.tabs.file-links.saving',
		defaultMessage: 'Applying...',
	},
	helper: {
		id: 'instance.settings.tabs.file-links.helper',
		defaultMessage:
			'Saving will fail if the instance path already exists and is not already a link. Targets must already exist outside the instance folder.',
	},
	selectFolder: {
		id: 'instance.settings.tabs.file-links.select-folder',
		defaultMessage: 'Select a folder to link',
	},
	selectFile: {
		id: 'instance.settings.tabs.file-links.select-file',
		defaultMessage: 'Select a file to link',
	},
})

function createDraft(fileLink?: Partial<FileLink>): FileLinkDraft {
	return {
		id: nextDraftId++,
		path: fileLink?.path ?? '',
		target: fileLink?.target ?? '',
		kind: fileLink?.kind ?? 'directory',
	}
}

function resetDraftLinks(fileLinks: FileLink[] = []) {
	draftLinks.value = fileLinks.map((fileLink) => createDraft(fileLink))
}

function serializeLinks(fileLinks: FileLink[]): string {
	return JSON.stringify(
		fileLinks.map((fileLink) => ({
			path: fileLink.path.trim(),
			target: fileLink.target.trim(),
			kind: fileLink.kind,
		})),
	)
}

const sanitizedDraftLinks = computed<FileLink[]>(() =>
	draftLinks.value.map(({ path, target, kind }) => ({
		path: path.trim(),
		target: target.trim(),
		kind,
	})),
)

const hasInvalidLinks = computed(() =>
	sanitizedDraftLinks.value.some((fileLink) => !fileLink.path || !fileLink.target),
)

const hasChanges = computed(
	() =>
		serializeLinks(sanitizedDraftLinks.value) !==
		serializeLinks(instance.value.file_links ?? []),
)

watch(
	() => [instance.value.path, serializeLinks(instance.value.file_links ?? [])],
	() => {
		if (!saving.value) {
			resetDraftLinks(instance.value.file_links ?? [])
		}
	},
	{ immediate: true },
)

function addLink() {
	draftLinks.value.push(createDraft())
}

function removeLink(id: number) {
	draftLinks.value = draftLinks.value.filter((draft) => draft.id !== id)
}

function setDraftKind(id: number, kind: FileLinkKind) {
	const draft = draftLinks.value.find((candidate) => candidate.id === id)
	if (draft) {
		draft.kind = kind
	}
}

async function browseTarget(id: number) {
	const draft = draftLinks.value.find((candidate) => candidate.id === id)
	if (!draft) return

	const result = await open({
		multiple: false,
		directory: draft.kind === 'directory',
		title:
			draft.kind === 'directory'
				? formatMessage(messages.selectFolder)
				: formatMessage(messages.selectFile),
	})

	if (typeof result === 'string') {
		draft.target = result
	}
}

async function saveLinks() {
	saving.value = true

	try {
		const savedLinks = await set_file_links(instance.value.path, sanitizedDraftLinks.value)
		resetDraftLinks(savedLinks)
		addNotification({
			title: formatMessage(commonMessages.changesSavedLabel),
			type: 'success',
		})
	} catch (error) {
		handleError(error)
	} finally {
		saving.value = false
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.description) }}
			</p>
		</div>

		<div v-if="draftLinks.length === 0" class="rounded-2xl border border-solid border-surface-5 p-4">
			<div class="flex items-center gap-3 text-secondary">
				<LinkIcon class="size-5" />
				<span>{{ formatMessage(messages.noLinks) }}</span>
			</div>
		</div>

		<div v-for="draft in draftLinks" :key="draft.id" class="file-link-card">
			<div class="flex items-center justify-between gap-2">
				<div class="flex items-center gap-2 text-contrast">
					<LinkIcon class="size-4" />
					<span class="font-semibold">{{ draft.path || formatMessage(messages.title) }}</span>
				</div>
				<ButtonStyled circular type="transparent" color="red">
					<button aria-label="Remove file link" @click="removeLink(draft.id)">
						<TrashIcon class="size-4" />
					</button>
				</ButtonStyled>
			</div>

			<div class="grid gap-4 md:grid-cols-2">
				<div class="flex flex-col gap-2">
					<label class="text-sm font-medium text-secondary">
						{{ formatMessage(messages.pathLabel) }}
					</label>
					<StyledInput
						v-model="draft.path"
						:placeholder="formatMessage(messages.pathPlaceholder)"
						wrapper-class="w-full"
					/>
				</div>

				<div class="flex flex-col gap-2">
					<label class="text-sm font-medium text-secondary">
						{{ formatMessage(messages.targetLabel) }}
					</label>
					<div class="flex gap-2">
						<StyledInput
							v-model="draft.target"
							:placeholder="formatMessage(messages.targetPlaceholder)"
							wrapper-class="min-w-0 flex-1"
						/>
						<ButtonStyled type="outlined">
							<button class="!h-10 whitespace-nowrap" @click="browseTarget(draft.id)">
								<FolderSearchIcon class="size-4" />
								{{ formatMessage(messages.browseTarget) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>

			<div class="flex flex-wrap items-center gap-2">
				<ButtonStyled :color="draft.kind === 'directory' ? 'brand' : 'standard'">
					<button class="!h-9" @click="setDraftKind(draft.id, 'directory')">
						<FolderSearchIcon class="size-4" />
						{{ formatMessage(messages.folderType) }}
					</button>
				</ButtonStyled>
				<ButtonStyled :color="draft.kind === 'file' ? 'brand' : 'standard'">
					<button class="!h-9" @click="setDraftKind(draft.id, 'file')">
						<FileIcon class="size-4" />
						{{ formatMessage(messages.fileType) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<ButtonStyled type="outlined">
				<button class="!h-10" @click="addLink">
					<PlusIcon class="size-4" />
					{{ formatMessage(messages.addLink) }}
				</button>
			</ButtonStyled>

			<ButtonStyled>
				<button class="!h-10" :disabled="saving || hasInvalidLinks || !hasChanges" @click="saveLinks">
					<SpinnerIcon v-if="saving" class="size-4 animate-spin" />
					<SaveIcon v-else class="size-4" />
					{{ formatMessage(saving ? messages.savingLinks : messages.saveLinks) }}
				</button>
			</ButtonStyled>
		</div>

		<p class="m-0 leading-tight text-secondary">
			{{ formatMessage(messages.helper) }}
		</p>
	</div>
</template>

<style scoped lang="scss">
.file-link-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	padding: 1rem;
	border: 1px solid var(--color-surface-5);
	border-radius: 1rem;
	background: var(--color-bg-raised);
}
</style>
