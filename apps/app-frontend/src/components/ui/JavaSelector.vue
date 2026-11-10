<template>
	<JavaDetectionModal ref="detectJavaModal" @submit="(val) => emit('update:modelValue', val)" />
	<div
		:id="id"
		class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
	>
		<Input
			v-if="editingPath"
			ref="pathInput"
			v-model="pathDraft"
			autocomplete="off"
			:placeholder="placeholder ?? formatMessage(messages.pathPlaceholder)"
			wrapper-class="w-full"
			@keydown.enter="commitPath"
			@keydown.esc="editingPath = false"
			@blur="commitPath"
		/>
		<Button
			v-else
			v-tooltip="path ? formatMessage(messages.copyPath) : undefined"
			size="lg"
			class="w-full !justify-between text-left"
			:disabled="!path"
			@click="copyPath"
		>
			<span
				class="min-w-0 truncate text-base font-semibold"
				:class="path ? 'text-primary' : 'text-secondary'"
			>
				{{ path || formatMessage(messages.notSet) }}
			</span>
			<ClipboardCopyIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
		</Button>

		<div
			class="flex flex-wrap items-center gap-2"
			:class="status === 'empty' ? 'justify-end' : 'justify-between'"
		>
			<span v-if="status === 'checking'" class="flex items-center gap-1.5 text-secondary">
				<SpinnerIcon class="size-4 animate-spin" aria-hidden="true" />
				{{ formatMessage(messages.checking) }}
			</span>
			<span
				v-else-if="status !== 'empty'"
				class="flex items-center gap-1.5 font-medium"
				:class="status === 'valid' ? 'text-green' : 'text-red'"
			>
				<CheckCircleIcon v-if="status === 'valid'" class="size-4" aria-hidden="true" />
				<XCircleIcon v-else class="size-4" aria-hidden="true" />
				{{ formatMessage(status === 'valid' ? messages.valid : messages.invalid) }}
			</span>

			<div class="flex flex-wrap items-center gap-2">
				<Button
					v-if="version && (status === 'empty' || status === 'invalid')"
					type="colored"
					color="brand"
					:disabled="disabled || installingJava"
					@click="installRecommended"
				>
					<SpinnerIcon v-if="installingJava" class="animate-spin" aria-hidden="true" />
					<DownloadIcon v-else aria-hidden="true" />
					{{
						installingJava
							? formatMessage(messages.installing)
							: formatMessage(messages.installJava, { version })
					}}
				</Button>
				<SplitButton
					:menu-label="formatMessage(messages.moreOptions)"
					:options="menuOptions"
					:disabled="disabled"
					@click="detectJavaModal?.show(version, modelValue)"
				>
					<SearchIcon aria-hidden="true" />
					{{ formatMessage(messages.detect) }}
				</SplitButton>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	FolderSearchIcon,
	SearchIcon,
	SpinnerIcon,
	XCircleIcon,
} from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	defineMessages,
	injectNotificationManager,
	Input,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, ref, watch } from 'vue'

import JavaDetectionModal from '@/components/ui/JavaDetectionModal.vue'
import useJavaTest from '@/composables/useJavaTest'
import { trackEvent } from '@/helpers/analytics'
import type { JavaVersion } from '@/helpers/instance'
import { auto_install_java, get_jre } from '@/helpers/jre.js'

const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	pathPlaceholder: {
		id: 'app.java-selector.path.placeholder',
		defaultMessage: '/path/to/java',
	},
	notSet: {
		id: 'app.java-selector.not-set',
		defaultMessage: 'No Java selected',
	},
	copyPath: {
		id: 'app.java-selector.copy-path',
		defaultMessage: 'Copy path',
	},
	pathCopiedTitle: {
		id: 'app.java-selector.path-copied-title',
		defaultMessage: 'Path copied',
	},
	pathCopiedText: {
		id: 'app.java-selector.path-copied-text',
		defaultMessage: 'The Java path has been copied to your clipboard.',
	},
	copyFailedTitle: {
		id: 'app.java-selector.copy-failed-title',
		defaultMessage: 'Failed to copy path',
	},
	checking: {
		id: 'app.java-selector.checking',
		defaultMessage: 'Checking Java…',
	},
	valid: {
		id: 'app.java-selector.valid',
		defaultMessage: 'Valid',
	},
	invalid: {
		id: 'app.java-selector.invalid',
		defaultMessage: 'Invalid',
	},
	installJava: {
		id: 'app.java-selector.install-java',
		defaultMessage: 'Install Java {version}',
	},
	installing: {
		id: 'app.java-selector.installing',
		defaultMessage: 'Installing...',
	},
	browse: {
		id: 'app.java-selector.browse',
		defaultMessage: 'Browse',
	},
	moreOptions: {
		id: 'app.java-selector.more-options',
		defaultMessage: 'More Java options',
	},
	detect: {
		id: 'app.java-selector.detect',
		defaultMessage: 'Detect',
	},
	editPath: {
		id: 'app.java-selector.edit-path',
		defaultMessage: 'Edit path',
	},
})

type JavaSelection = Partial<JavaVersion>

const props = withDefaults(
	defineProps<{
		id?: string
		version?: number | null
		modelValue?: JavaSelection | null
		disabled?: boolean
		placeholder?: string
	}>(),
	{
		id: undefined,
		version: null,
		modelValue: null,
		disabled: false,
		placeholder: undefined,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: JavaSelection]
}>()

const { status, checkJava } = useJavaTest()

const path = computed(() => props.modelValue?.path ?? '')
const installingJava = ref(false)
const editingPath = ref(false)
const pathDraft = ref('')
const pathInput = ref<InstanceType<typeof Input>>()
const detectJavaModal = ref<InstanceType<typeof JavaDetectionModal>>()

watch(path, (newPath) => checkJava(newPath, props.version), { immediate: true })

const menuOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'install',
		label: formatMessage(messages.installJava, { version: props.version }),
		icon: DownloadIcon,
		shown: !!props.version && status.value === 'valid',
		disabled: installingJava.value,
		action: installRecommended,
	},
	{
		id: 'browse',
		label: formatMessage(messages.browse),
		icon: FolderSearchIcon,
		action: browse,
	},
	{
		id: 'enter-path',
		label: formatMessage(messages.editPath),
		icon: EditIcon,
		action: startEditingPath,
	},
])

function fallbackSelection(selectedPath: string): JavaSelection {
	return {
		path: selectedPath,
		version: props.version?.toString() ?? '',
		parsed_version: props.version ?? undefined,
		architecture: 'x86',
	}
}

async function selectPath(selectedPath: string) {
	const result: JavaVersion | null = await get_jre(selectedPath).catch(() => null)
	emit('update:modelValue', result ?? fallbackSelection(selectedPath))
}

async function browse() {
	const selected = await open()
	if (!selected) return

	trackEvent('JavaManualSelect', { version: props.version?.toString() ?? '' })
	await selectPath(selected)
}

function startEditingPath() {
	pathDraft.value = path.value
	editingPath.value = true
	setTimeout(() => {
		pathInput.value?.focus()
		pathInput.value?.select()
	})
}

async function commitPath() {
	if (!editingPath.value) return
	editingPath.value = false

	const newPath = pathDraft.value.trim()
	if (newPath === path.value) return
	if (!newPath) {
		emit('update:modelValue', { ...props.modelValue, path: '' })
		return
	}
	await selectPath(newPath)
}

async function copyPath() {
	try {
		await navigator.clipboard.writeText(path.value)
		addNotification({
			type: 'success',
			title: formatMessage(messages.pathCopiedTitle),
			text: formatMessage(messages.pathCopiedText),
		})
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.copyFailedTitle),
			text: error instanceof Error ? error.message : String(error),
		})
	}
}

async function installRecommended() {
	if (!props.version || installingJava.value) return

	installingJava.value = true
	try {
		const installedPath: string | undefined = await auto_install_java(props.version).catch(
			handleError,
		)
		if (installedPath) await selectPath(installedPath)
	} finally {
		installingJava.value = false
	}
}
</script>
