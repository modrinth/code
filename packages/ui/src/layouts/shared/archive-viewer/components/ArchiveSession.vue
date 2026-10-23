<template>
	<ArchiveScope
		v-if="stage === 'ready' && root && handler && loadedSource"
		:key="root.key"
		:root="root"
		:handler="handler"
		:source="loadedSource"
	/>
	<slot :opened="opened" />
</template>

<script setup lang="ts">
import { computed, ref, shallowRef, toRef, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { provideFileManager } from '#ui/layouts/shared/files-tab/providers/file-manager'
import {
	emptyFileManager,
	switchableFileManager,
} from '#ui/layouts/shared/files-tab/providers/switchable-file-manager'

import { useArchiveLoader } from '../composables/archive-loader'
import { type OpenArchive, provideArchiveSession } from '../providers/archive-session'
import {
	injectArchiveSettings,
	provideArchiveSettings,
	useStoredArchiveSettings,
} from '../settings'
import type { ArchiveSource } from '../types'
import ArchiveScope from './archive-scope'

/**
 * Opens an archive for everything within it: the files layout and any workbench showing the
 * archive's panels. Opening another archive swaps what they browse, leaving them mounted.
 */
const props = defineProps<{
	source: ArchiveSource | null
	/** Path of the entry open within the archive, without a leading slash. */
	path?: string | null
	/** Keeps archives open in the worker once they are left, so reopening them skips reading them. */
	retain?: boolean
}>()

const emit = defineEmits<{
	'update:path': [path: string]
}>()

defineSlots<{
	default?: (props: { opened: boolean }) => unknown
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	readOnly: {
		id: 'archive-viewer.read-only',
		defaultMessage: 'Archives are read-only',
	},
})

provideArchiveSettings(injectArchiveSettings(null) ?? useStoredArchiveSettings())

const { stage, progress, error, root, handler, fullyLoaded, retry } = useArchiveLoader(
	toRef(props, 'source'),
	{ retain: props.retain },
)

/** The source the archive that was read came from, which `source` may have moved on from. */
const loadedSource = shallowRef<ArchiveSource | null>(null)
watch(root, () => (loadedSource.value = root.value ? props.source : null), { immediate: true })

const current = shallowRef<OpenArchive | null>(null)
const opened = ref(false)

provideFileManager(
	switchableFileManager(
		computed(() => current.value?.manager ?? null),
		emptyFileManager({ readOnlyReason: computed(() => formatMessage(messages.readOnly)) }),
	),
)

provideArchiveSession({
	source: toRef(props, 'source'),
	stage,
	progress,
	error,
	fullyLoaded,
	retry,
	current,
	opened,
	initialPath: () => props.path ?? null,
	attach: (archive) => {
		current.value = archive
		opened.value = true
	},
	detach: (archive) => {
		if (current.value === archive) current.value = null
	},
})

watch(
	() => current.value?.location.value,
	(location) => {
		if (location) emit('update:path', location.path.replace(/^\/+/, ''))
	},
)

watch(
	() => props.path,
	(path) => current.value?.open(path ?? ''),
)
</script>
