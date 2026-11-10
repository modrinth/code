<template>
	<SharedFilesPublishModal
		ref="modal"
		:diffs="diffs"
		:files="filesQuery.data.value?.files ?? []"
		:changes-description="changesDescription"
		:confirm-label="confirmLabel"
		:restart-confirm-label="restartConfirmLabel"
		:requires-restart="requiresRestart"
		:restart-disabled="!canUsePowerActions"
		allow-empty
		:disabled="disabled"
		:changes-loading="loading"
		:changes-error="error"
		:files-loading="filesQuery.isFetching.value && !filesQuery.data.value"
		:files-error="filesQuery.isError.value"
		@confirm="confirm"
		@cancel="cancel"
		@retry-changes="emit('retry')"
		@retry-files="filesQuery.refetch()"
	/>
</template>

<script setup lang="ts">
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import SharedFilesPublishModal from '#ui/components/sharing/SharedFilesPublishModal.vue'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { serverShareableFilesQueryOptions } from '#ui/composables/server-sharing/query-options'
import type { ContentDiffItem } from '#ui/layouts/shared/installation-settings/types'
import { injectAuth, injectModrinthClient } from '#ui/providers'

const props = defineProps<{
	serverId: string
	worldId?: string
	diffs: ContentDiffItem[]
	changesDescription?: string
	confirmLabel?: string
	restartConfirmLabel?: string
	requiresRestart?: boolean
	loading?: boolean
	error?: boolean
	disabled?: boolean
}>()

const emit = defineEmits<{
	confirm: [paths: string[], restart: boolean]
	cancel: []
	retry: []
}>()

const client = injectModrinthClient()
const auth = injectAuth()
const { canUsePowerActions } = useServerPermissions()
const modal = ref<InstanceType<typeof SharedFilesPublishModal>>()
const open = ref(false)
const filesQuery = useQuery(
	computed(() => ({
		...serverShareableFilesQueryOptions(
			client,
			props.serverId,
			props.worldId ?? '',
			auth.user.value?.id,
		),
		enabled: open.value && !!props.worldId,
	})),
)

/** Opens the review. Pass `keepSelection` to reopen it after a failed push without losing the picked files. */
function show(options?: { keepSelection?: boolean }) {
	open.value = true
	modal.value?.show(options)
}

function hide() {
	modal.value?.hide()
}

function confirm(paths: string[], restart: boolean) {
	open.value = false
	emit('confirm', paths, restart)
}

function cancel() {
	open.value = false
	emit('cancel')
}

defineExpose({ show, hide })
</script>
