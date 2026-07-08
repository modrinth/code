<template>
	<Admonition
		type="warning"
		inline-actions
		:header="formatMessage(messages.sharedInstanceChangesHeader)"
	>
		{{ formatMessage(messages.sharedInstanceChangesBody) }}
		<template #actions>
			<ButtonStyled color="orange">
				<button class="!h-10" :disabled="isPublishButtonDisabled" @click="reviewChanges">
					<UploadIcon aria-hidden="true" />
					{{
						isPublishing
							? formatMessage(messages.sharedInstancePublishingButton)
							: isReviewingPublish
								? formatMessage(messages.sharedInstanceReviewingButton)
								: formatMessage(messages.sharedInstancePublishButton)
					}}
				</button>
			</ButtonStyled>
		</template>
	</Admonition>

	<ContentDiffModal
		ref="publishReviewModal"
		:header="formatMessage(messages.sharedInstanceReviewHeader)"
		:admonition-header="formatMessage(messages.sharedInstanceReviewAdmonitionHeader)"
		:description="formatMessage(messages.sharedInstanceReviewDescription)"
		:diffs="publishDiffs"
		:confirm-label="formatMessage(messages.sharedInstancePublishButton)"
		:confirm-icon="UploadIcon"
		:added-label="formatMessage(messages.sharedInstanceAddedLabel)"
		:removed-label="formatMessage(messages.sharedInstanceRemovedLabel)"
		@confirm="publishChanges"
	/>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	type ContentDiffItem,
	ContentDiffModal,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getErrorMessage,
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

import {
	instanceAdmonitionsMessages as messages,
	sharedInstanceUnavailableTextMessage,
} from './instance-admonitions-messages'

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const isPublishing = ref(false)
const isReviewingPublish = ref(false)
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)

function notifySharedInstanceUnavailable(reason: SharedInstanceUnavailableReason | null = null) {
	addNotification({
		type: 'warning',
		title: formatMessage(messages.sharedInstanceUnavailableTitle),
		text: formatMessage(sharedInstanceUnavailableTextMessage(reason), {
			manager: formatMessage(messages.sharedInstanceUnavailableFallbackManager),
		}),
	})
}

function notifySharedInstanceError(error: unknown) {
	addNotification({
		type: 'error',
		title: formatMessage(messages.sharedInstanceErrorTitle),
		text: getErrorMessage(error),
	})
}

async function reviewChanges(e?: MouseEvent) {
	if (isPublishButtonDisabled.value) return

	isReviewingPublish.value = true
	try {
		const preview = await get_shared_instance_publish_preview(props.instance.id)
		if (!preview) {
			notifySharedInstanceUnavailable()
			emit('published')
			return
		}
		if (preview.diffs.length === 0) {
			emit('published')
			return
		}

		publishDiffs.value = preview.diffs.map((diff) => ({
			type: diff.type,
			projectName: diff.projectName ?? undefined,
			fileName: diff.fileName ?? undefined,
			currentVersionName: diff.currentVersionName ?? undefined,
			newVersionName: diff.newVersionName ?? undefined,
			disabled: diff.disabled,
		}))
		publishReviewModal.value?.show(e)
	} catch (err) {
		if (isSharedInstanceUnavailableError(err)) {
			notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(err))
			emit('published')
			return
		}

		notifySharedInstanceError(err)
	} finally {
		isReviewingPublish.value = false
	}
}

async function publishChanges() {
	if (isPublishing.value) return

	isPublishing.value = true
	try {
		await publish_shared_instance(props.instance.id)
		emit('published')
	} catch (err) {
		if (isSharedInstanceUnavailableError(err)) {
			notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(err))
			emit('published')
			return
		}

		notifySharedInstanceError(err)
	} finally {
		isPublishing.value = false
	}
}
</script>
