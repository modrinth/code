<template>
	<JavaDetectionModal ref="detectJavaModal" @submit="selectJre" />
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
			:class="status === 'valid' || status === 'invalid' ? 'justify-between' : 'justify-end'"
		>
			<span
				v-if="status === 'valid' || status === 'invalid'"
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
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import { jreQueryOptions } from '@/features/java/queries'
import { trackEvent } from '@/helpers/analytics'
import { autoInstallJava } from '@/platform/app-lib/jre/commands'
import type { JavaVersion } from '@/platform/app-lib/jre/types'
import { pickFile } from '@/platform/dialogs'

import JavaDetectionModal from './detection-modal.vue'

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

const queryClient = useQueryClient()

const path = computed(() => props.modelValue?.path ?? '')
const editingPath = ref(false)
const pathDraft = ref('')
const pathInput = ref<InstanceType<typeof Input>>()
const detectJavaModal = ref<InstanceType<typeof JavaDetectionModal>>()

const { data: jre } = useQuery(
	computed(() => ({ ...jreQueryOptions(path.value), enabled: !!path.value })),
)

const installJavaMutation = useMutation({
	mutationFn: autoInstallJava,
	onSuccess: (installedPath) => selectPath(installedPath),
	onError: handleError,
})
const installingJava = computed(() => installJavaMutation.isPending.value)

const status = computed<'empty' | 'valid' | 'invalid' | null>(() => {
	if (!path.value) return 'empty'
	if (jre.value === undefined) return null
	return jre.value && (props.version === null || jre.value.parsed_version === props.version)
		? 'valid'
		: 'invalid'
})

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

function selectJre(selected: JavaVersion) {
	queryClient.setQueryData(jreQueryOptions(selected.path).queryKey, selected)
	emit('update:modelValue', selected)
}

async function selectPath(selectedPath: string) {
	const result = await queryClient.fetchQuery({ ...jreQueryOptions(selectedPath), staleTime: 0 })
	emit('update:modelValue', result ?? fallbackSelection(selectedPath))
}

async function browse() {
	const selected = await pickFile()
	if (!selected) return

	trackEvent('JavaManualSelect', { version: props.version?.toString() ?? '' })
	await selectPath(selected)
}

function startEditingPath() {
	pathDraft.value = path.value
	editingPath.value = true
	nextTick(() => {
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

function installRecommended() {
	if (!props.version || installingJava.value) return
	installJavaMutation.mutate(props.version)
}
</script>
