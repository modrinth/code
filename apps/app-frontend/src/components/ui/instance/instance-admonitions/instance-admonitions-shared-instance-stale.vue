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
					<SpinnerIcon
						v-if="isReviewingPublish || isPublishing"
						class="animate-spin"
						aria-hidden="true"
					/>
					<UploadIcon v-else aria-hidden="true" />
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
		@cancel="finishReview"
	/>
</template>

<script setup lang="ts">
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	type ContentDiffItem,
	ContentDiffModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'

import { instanceAdmonitionsMessages as messages } from './instance-admonitions-messages'

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const { formatMessage } = useVIntl()
const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()
const isPublishing = ref(false)
const isReviewingPublish = ref(false)
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)

async function reviewChanges(e?: MouseEvent) {
	if (isPublishButtonDisabled.value) return

	isReviewingPublish.value = true
	let reviewOpened = false
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
			fileCount: diff.configFileCount ?? undefined,
			disabled: diff.disabled,
		}))
		if (!publishReviewModal.value) return

		publishReviewModal.value.show(e)
		reviewOpened = true
	} catch (err) {
		if (isSharedInstanceUnavailableError(err)) {
			notifySharedInstanceUnavailable(getSharedInstanceUnavailableReason(err))
			emit('published')
			return
		}

		notifySharedInstanceError(err)
	} finally {
		if (!reviewOpened) finishReview()
	}
}

async function publishChanges() {
	finishReview()
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

function finishReview() {
	isReviewingPublish.value = false
}
</script>
