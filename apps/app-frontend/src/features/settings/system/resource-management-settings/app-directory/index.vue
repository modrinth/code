<script setup lang="ts">
import {
	ClipboardCopyIcon,
	FolderOpenIcon,
	FolderSearchIcon,
	RefreshCwIcon,
	UndoIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Button,
	type ButtonMenuOption,
	defineMessages,
	injectNotificationManager,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import { getOS, openPath } from '@/helpers/utils.js'

import ChangeDirectoryModal from './change-directory-modal.vue'
import { normalizePath } from './paths'
import { useAppDirectory } from './use-app-directory'
import { useCopyPath } from './use-copy-path'

const { formatMessage } = useVIntl()
const copyPath = useCopyPath()
const { handleError } = injectNotificationManager()
const { currentDir, defaultDir, pendingDir, busy, changeDirectory, cancelMove, restartNow } =
	useAppDirectory()

const messages = defineMessages({
	title: {
		id: 'app.settings.resource-management.app-directory.title',
		defaultMessage: 'App directory',
	},
	description: {
		id: 'app.settings.resource-management.app-directory.description',
		defaultMessage: 'Where Modrinth App stores instances and other files.',
	},
	pending: {
		id: 'app.settings.resource-management.app-directory.pending',
		defaultMessage: 'Your app directory will move to {path} the next time Modrinth App opens.',
	},
	cancelMove: {
		id: 'app.settings.resource-management.app-directory.cancel-move',
		defaultMessage: 'Cancel move',
	},
	moveNow: {
		id: 'app.settings.resource-management.app-directory.move-now',
		defaultMessage: 'Move and restart now',
	},
	openInFinder: {
		id: 'app.settings.resource-management.app-directory.open-in-finder',
		defaultMessage: 'Open in Finder',
	},
	openInExplorer: {
		id: 'app.settings.resource-management.app-directory.open-in-explorer',
		defaultMessage: 'Open in File Explorer',
	},
	openFolder: {
		id: 'app.settings.resource-management.app-directory.open-folder',
		defaultMessage: 'Open folder',
	},
	changeDirectory: {
		id: 'app.settings.resource-management.app-directory.change',
		defaultMessage: 'Change directory',
	},
	changeToDefault: {
		id: 'app.settings.resource-management.app-directory.change-to-default',
		defaultMessage: 'Change to default',
	},
	moreChangeOptions: {
		id: 'app.settings.resource-management.app-directory.more-change-options',
		defaultMessage: 'More directory options',
	},
})

const changeModal = ref<InstanceType<typeof ChangeDirectoryModal>>()
const os = ref<string | null>(null)

onMounted(async () => {
	os.value = await getOS()
})

const isDefaultDir = computed(() => {
	if (!currentDir.value || !defaultDir.value) return true
	const caseInsensitive = os.value === 'Windows'
	return (
		normalizePath(currentDir.value, caseInsensitive) ===
		normalizePath(defaultDir.value, caseInsensitive)
	)
})

const changeOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'change-to-default',
		label: formatMessage(messages.changeToDefault),
		icon: UndoIcon,
		action: (event) => {
			if (defaultDir.value) changeModal.value?.show(event, defaultDir.value)
		},
	},
])

const openFolderLabel = computed(() => {
	if (os.value === 'MacOS') return formatMessage(messages.openInFinder)
	if (os.value === 'Windows') return formatMessage(messages.openInExplorer)
	return formatMessage(messages.openFolder)
})

function openCurrentDir() {
	if (currentDir.value) {
		openPath(currentDir.value).catch(handleError)
	}
}
</script>

<template>
	<section class="flex flex-col gap-2.5" aria-labelledby="app-directory-title">
		<div class="flex flex-col gap-1">
			<h2 id="app-directory-title" class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.title) }}
			</h2>
			<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		</div>

		<div
			class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
		>
			<Button
				v-if="currentDir"
				size="lg"
				class="w-full !justify-between text-left"
				@click="copyPath(currentDir)"
			>
				<span class="min-w-0 truncate text-base font-semibold text-primary">
					{{ currentDir }}
				</span>
				<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
			</Button>

			<Admonition
				v-if="pendingDir"
				type="warning"
				:body="formatMessage(messages.pending, { path: pendingDir })"
			>
				<template #actions>
					<div class="flex flex-wrap gap-2">
						<Button type="outlined" :disabled="busy" @click="cancelMove">
							<XIcon aria-hidden="true" />
							{{ formatMessage(messages.cancelMove) }}
						</Button>
						<Button type="colored" color="red" :disabled="busy" @click="restartNow">
							<RefreshCwIcon aria-hidden="true" />
							{{ formatMessage(messages.moveNow) }}
						</Button>
					</div>
				</template>
			</Admonition>

			<div class="flex flex-wrap gap-2">
				<Button :disabled="!currentDir" @click="openCurrentDir">
					<FolderOpenIcon aria-hidden="true" />
					{{ openFolderLabel }}
				</Button>
				<Button
					v-if="isDefaultDir"
					:disabled="!currentDir || busy"
					@click="changeModal?.show($event)"
				>
					<FolderSearchIcon aria-hidden="true" />
					{{ formatMessage(messages.changeDirectory) }}
				</Button>
				<SplitButton
					v-else
					:menu-label="formatMessage(messages.moreChangeOptions)"
					:options="changeOptions"
					:disabled="busy"
					@click="changeModal?.show($event)"
				>
					<FolderSearchIcon aria-hidden="true" />
					{{ formatMessage(messages.changeDirectory) }}
				</SplitButton>
			</div>
		</div>

		<ChangeDirectoryModal
			v-if="currentDir"
			ref="changeModal"
			:current-dir="currentDir"
			:case-insensitive="os === 'Windows'"
			@confirm="changeDirectory"
		/>
	</section>
</template>
