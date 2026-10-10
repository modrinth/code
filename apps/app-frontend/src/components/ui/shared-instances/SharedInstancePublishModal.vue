<template>
	<ContentDiffModal
		ref="publishReviewModal"
		:header="formatMessage(messages.header)"
		:admonition-header="formatMessage(messages.admonitionHeader)"
		:description="formatMessage(messages.description)"
		:diffs="publishDiffs"
		:confirm-label="
			formatMessage(hasServers ? messages.pushAndRestartButton : messages.publishButton)
		"
		:confirm-icon="UploadIcon"
		:secondary-confirm-label="hasServers ? formatMessage(messages.publishButton) : undefined"
		:confirm-disabled="publishDiffs.length === 0 && selectedConfigPaths.length === 0"
		:added-label="formatMessage(messages.addedLabel)"
		:removed-label="formatMessage(messages.removedLabel)"
		@confirm="publishChanges(hasServers)"
		@secondary-confirm="publishChanges(false)"
		@cancel="finishReview"
	>
		<template v-if="configFilePaths.length > 0" #additional-content>
			<Accordion
				class="min-w-0"
				overflow-visible
				button-class="w-full bg-transparent m-0 p-0 border-none"
			>
				<template #title>
					<FileCogIcon class="size-4 shrink-0 text-primary" />
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.configTitle) }}
					</span>
				</template>
				<div class="flex min-w-0 flex-col gap-3 pt-4">
					<div ref="configFileTreeContainer" class="max-h-[292px] overflow-y-auto rounded-[20px]">
						<FileTreeSelect
							v-model="includedConfigPaths"
							v-model:excluded-paths="excludedConfigPaths"
							:items="configFileItems"
							:show-size="false"
							:show-modified="false"
							@navigate="scrollConfigFileTreeToTop"
						/>
					</div>
				</div>
			</Accordion>
		</template>
	</ContentDiffModal>
</template>

<script setup lang="ts">
import { FileCogIcon, UploadIcon } from '@modrinth/assets'
import {
	Accordion,
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	FileTreeSelect,
	type FileTreeSelectItem,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
import { diffInstanceAgainstServer, type LinkedServer } from '@/helpers/linked-servers'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'

type SharedInstancePublishState = 'idle' | 'reviewing' | 'publishing'

/**
 * The review every push goes through. It lists what changed and only lets a push happen when
 * something did. A shared instance publishes to friends; an instance with linked servers also
 * offers "Push and restart" and emits `push-servers` for the caller to update them.
 */
const props = defineProps<{
	instance: GameInstance
	servers?: LinkedServer[]
}>()

const emit = defineEmits<{
	published: []
	'push-servers': [restart: boolean]
	'state-change': [state: SharedInstancePublishState]
}>()

const { formatMessage } = useVIntl()
const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const configFileTreeContainer = ref<HTMLElement>()
const publishDiffs = ref<ContentDiffItem[]>([])
const configFilePaths = ref<string[]>([])
const includedConfigPaths = ref<string[]>([])
const excludedConfigPaths = ref<string[]>([])
const state = ref<SharedInstancePublishState>('idle')
const hasServers = computed(() => (props.servers?.length ?? 0) > 0)
const sharesWithFriends = computed(() => props.instance.shared_instance?.role === 'owner')
// Whether the changes on screen are ones friends have not received yet.
const friendsBehind = ref(false)
const configFileItems = computed<FileTreeSelectItem[]>(() =>
	configFilePaths.value.map((path) => ({ path, type: 'file' })),
)
const selectedConfigPaths = computed(() => {
	const includedPaths = new Set(includedConfigPaths.value)
	const excludedPaths = new Set(excludedConfigPaths.value)

	return configFilePaths.value.filter((path) =>
		isConfigPathSelected(path, includedPaths, excludedPaths),
	)
})

async function show(e?: MouseEvent) {
	if (state.value !== 'idle') return

	setState('reviewing')
	let reviewOpened = false
	try {
		let friendDiffs: ContentDiffItem[] = []
		configFilePaths.value = []
		if (sharesWithFriends.value) {
			const preview = await get_shared_instance_publish_preview(props.instance.id)
			if (!preview) {
				notifySharedInstanceUnavailable()
				emit('published')
				if (!hasServers.value) return
			} else {
				friendDiffs = preview.diffs.map((diff) => ({
					type: diff.type,
					projectName: diff.projectName ?? undefined,
					fileName: diff.fileName ?? undefined,
					currentVersionName: diff.currentVersionName ?? undefined,
					newVersionName: diff.newVersionName ?? undefined,
					fileCount: diff.configFileCount ?? undefined,
					disabled: diff.disabled,
				}))
				configFilePaths.value = preview.configFiles
			}
		}
		friendsBehind.value = friendDiffs.length > 0
		// Friends can be up to date while a server is not, so fall back to what the server lacks.
		let serverDiffs: ContentDiffItem[] = []
		if (friendDiffs.length === 0) {
			for (const server of props.servers ?? []) {
				serverDiffs = await diffInstanceAgainstServer(props.instance.id, server)
				if (serverDiffs.length > 0) break
			}
		}
		publishDiffs.value = friendDiffs.length > 0 ? friendDiffs : serverDiffs
		includedConfigPaths.value = []
		excludedConfigPaths.value = []
		if (!publishReviewModal.value) return

		publishReviewModal.value.show(e)
		reviewOpened = true
	} catch (error) {
		handlePublishError(error)
	} finally {
		if (!reviewOpened) finishReview()
	}
}

async function publishChanges(restart: boolean) {
	if (state.value !== 'reviewing') return

	setState('publishing')
	try {
		if (sharesWithFriends.value && (friendsBehind.value || selectedConfigPaths.value.length > 0)) {
			await publish_shared_instance(props.instance.id, selectedConfigPaths.value)
			emit('published')
		}
		if (hasServers.value) emit('push-servers', restart)
	} catch (error) {
		handlePublishError(error)
	} finally {
		setState('idle')
	}
}

function isConfigPathSelected(
	path: string,
	includedPaths: Set<string>,
	excludedPaths: Set<string>,
) {
	let selected = false
	let prefix = ''

	for (const segment of path.split('/').filter(Boolean)) {
		prefix = prefix ? `${prefix}/${segment}` : segment
		if (includedPaths.has(prefix)) selected = true
		if (excludedPaths.has(prefix)) selected = false
	}

	return selected
}

function scrollConfigFileTreeToTop() {
	if (configFileTreeContainer.value) {
		configFileTreeContainer.value.scrollTop = 0
	}
}

function finishReview() {
	if (state.value === 'reviewing') {
		setState('idle')
	}
}

function handlePublishError(error: unknown) {
	if (isSharedInstanceUnavailableError(error)) {
		notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(error))
		emit('published')
		return
	}

	notifySharedInstanceError(error)
}

function setState(nextState: SharedInstancePublishState) {
	state.value = nextState
	emit('state-change', nextState)
}

const messages = defineMessages({
	header: {
		id: 'instance.shared-instance.publish-review.header',
		defaultMessage: 'Review changes',
	},
	admonitionHeader: {
		id: 'instance.shared-instance.publish-review.admonition-header',
		defaultMessage: 'Push update',
	},
	description: {
		id: 'instance.shared-instance.publish-review.description',
		defaultMessage:
			'Review the content changes and choose any config files to include in this update.',
	},
	publishButton: {
		id: 'instance.shared-instance.publish-review.publish-button',
		defaultMessage: 'Push update',
	},
	pushAndRestartButton: {
		id: 'instance.shared-instance.publish-review.push-and-restart-button',
		defaultMessage: 'Push and restart',
	},
	addedLabel: {
		id: 'instance.shared-instance.publish-review.added-label',
		defaultMessage: 'Added',
	},
	removedLabel: {
		id: 'instance.shared-instance.publish-review.removed-label',
		defaultMessage: 'Removed',
	},
	configTitle: {
		id: 'instance.shared-instance.publish-review.config-title-v2',
		defaultMessage: 'Select config files',
	},
})

defineExpose({ show })
</script>
