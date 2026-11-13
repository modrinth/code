<script setup lang="ts">
import {
	ClipboardCopyIcon,
	ClockIcon,
	FolderSearchIcon,
	RefreshCwIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Button,
	type ButtonMenuOption,
	commonMessages,
	defineMessages,
	NewModal,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { get_all as getRunningProcesses } from '@/helpers/process.js'
import { pickDirectory } from '@/platform/dialogs'

import { isDriveRoot, isNestedPath, normalizePath } from './paths'
import { useCopyPath } from './use-copy-path'

const props = defineProps<{
	currentDir: string
	caseInsensitive: boolean
}>()

const emit = defineEmits<{
	confirm: [directory: string, restartNow: boolean]
}>()

const { formatMessage } = useVIntl()
const copyPath = useCopyPath()

const messages = defineMessages({
	header: {
		id: 'app.settings.resource-management.app-directory.modal.header',
		defaultMessage: 'Change app directory',
	},
	admonitionHeader: {
		id: 'app.settings.resource-management.app-directory.modal.admonition-header',
		defaultMessage: 'All your files will be moved',
	},
	admonitionBody: {
		id: 'app.settings.resource-management.app-directory.modal.admonition-body',
		defaultMessage:
			"You won't be able to use Modrinth App while we move your data. This may take a while.",
	},
	currentLabel: {
		id: 'app.settings.resource-management.app-directory.modal.current',
		defaultMessage: 'Current',
	},
	newLabel: {
		id: 'app.settings.resource-management.app-directory.modal.new',
		defaultMessage: 'New',
	},
	choose: {
		id: 'app.settings.resource-management.app-directory.modal.choose',
		defaultMessage: 'Choose a folder',
	},
	pickerTitle: {
		id: 'app.settings.resource-management.app-directory.modal.picker-title',
		defaultMessage: 'Select a new app directory',
	},
	instancesRunning: {
		id: 'app.settings.resource-management.app-directory.modal.instances-running',
		defaultMessage: 'Close all running instances before changing the app directory.',
	},
	nested: {
		id: 'app.settings.resource-management.app-directory.modal.nested',
		defaultMessage: "The new folder can't be inside your current app directory, or contain it.",
	},
	driveRoot: {
		id: 'app.settings.resource-management.app-directory.modal.drive-root',
		defaultMessage:
			"Choose a folder rather than the top of a drive, so your files don't mix with everything else on it.",
	},
	moveAndRestart: {
		id: 'app.settings.resource-management.app-directory.modal.move-and-restart',
		defaultMessage: 'Move and restart',
	},
	moveLater: {
		id: 'app.settings.resource-management.app-directory.modal.move-later',
		defaultMessage: 'Move on next launch',
	},
	moreOptions: {
		id: 'app.settings.resource-management.app-directory.modal.more-options',
		defaultMessage: 'More move options',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const newDir = ref<string | null>(null)
const locked = ref(false)

const processesQuery = useQuery({
	queryKey: ['processes', 'list'],
	networkMode: 'always',
	queryFn: (): Promise<unknown[]> => getRunningProcesses(),
	staleTime: 0,
})
const instancesRunning = computed(() => (processesQuery.data.value?.length ?? 0) > 0)

const normalizedNew = computed(() =>
	newDir.value === null ? null : normalizePath(newDir.value, props.caseInsensitive),
)
const normalizedCurrent = computed(() => normalizePath(props.currentDir, props.caseInsensitive))
const sameAsCurrent = computed(() => normalizedNew.value === normalizedCurrent.value)
const nested = computed(
	() =>
		normalizedNew.value !== null &&
		!sameAsCurrent.value &&
		isNestedPath(normalizedNew.value, normalizedCurrent.value),
)
const driveRoot = computed(() => newDir.value !== null && isDriveRoot(newDir.value))
const canMove = computed(
	() => newDir.value !== null && !sameAsCurrent.value && !nested.value && !instancesRunning.value,
)

const moveOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'move-later',
		label: formatMessage(messages.moveLater),
		icon: ClockIcon,
		action: () => confirm(false),
	},
])

function show(event?: MouseEvent, directory?: string) {
	newDir.value = directory ?? null
	locked.value = directory !== undefined
	void processesQuery.refetch()
	modal.value?.show(event)
}

async function browse() {
	const directory = await pickDirectory(formatMessage(messages.pickerTitle))
	if (directory) {
		newDir.value = directory
	}
}

async function confirm(restartNow: boolean) {
	await processesQuery.refetch()
	if (!canMove.value || newDir.value === null) return
	modal.value?.hide()
	emit('confirm', newDir.value, restartNow)
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="danger" max-width="560px">
		<div class="flex flex-col gap-6">
			<Admonition
				type="critical"
				:header="formatMessage(messages.admonitionHeader)"
				:body="formatMessage(messages.admonitionBody)"
			/>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.currentLabel) }}</span>
				<Button size="lg" class="w-full !justify-between text-left" @click="copyPath(currentDir)">
					<span class="min-w-0 truncate text-base font-semibold text-primary">
						{{ currentDir }}
					</span>
					<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
				</Button>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.newLabel) }}</span>
				<Button
					size="lg"
					class="w-full !justify-between text-left"
					:disabled="locked"
					@click="browse"
				>
					<span
						class="min-w-0 truncate font-semibold"
						:class="newDir ? 'text-primary' : 'text-secondary'"
						:title="newDir ?? undefined"
					>
						{{ newDir ?? formatMessage(messages.choose) }}
					</span>
					<FolderSearchIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
				</Button>
				<div aria-live="polite" class="flex flex-col gap-1 empty:hidden">
					<p v-if="instancesRunning" class="m-0 text-sm text-red">
						{{ formatMessage(messages.instancesRunning) }}
					</p>
					<p v-if="nested" class="m-0 text-sm text-red">
						{{ formatMessage(messages.nested) }}
					</p>
					<p v-else-if="driveRoot" class="m-0 text-sm text-orange">
						{{ formatMessage(messages.driveRoot) }}
					</p>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<SplitButton
					type="colored"
					color="red"
					:menu-label="formatMessage(messages.moreOptions)"
					:options="moveOptions"
					:disabled="!canMove"
					@click="confirm(true)"
				>
					<RefreshCwIcon aria-hidden="true" />
					{{ formatMessage(messages.moveAndRestart) }}
				</SplitButton>
			</div>
		</template>
	</NewModal>
</template>
