<template>
	<StackedAdmonitions v-bind="$attrs" :items="stackItems" class="w-full">
		<template #item="{ item }">
			<Admonition
				v-if="item.kind === 'shared-instance-stale'"
				type="warning"
				inline-actions
				:header="formatMessage(messages.sharedInstanceChangesHeader)"
			>
				{{ formatMessage(messages.sharedInstanceChangesBody) }}
				<template #actions>
					<ButtonStyled color="orange">
						<button class="!h-10" :disabled="isPublishButtonDisabled" @click="reviewSharedInstanceChanges">
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
		</template>
	</StackedAdmonitions>

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
		@confirm="publishSharedInstanceChanges"
	/>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	injectNotificationManager,
	type StackedAdmonitionItem,
	StackedAdmonitions,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

defineOptions({
	inheritAttrs: false,
})

type InstanceAdmonitionItem = StackedAdmonitionItem & {
	kind: 'shared-instance-stale'
}

const props = defineProps<{
	instance: GameInstance
}>()

const emit = defineEmits<{
	published: []
}>()

const messages = defineMessages({
	sharedInstanceChangesHeader: {
		id: 'app.instance.admonitions.shared-instance.changes-header',
		defaultMessage: "Your changes haven't been shared yet",
	},
	sharedInstanceChangesBody: {
		id: 'app.instance.admonitions.shared-instance.changes-body',
		defaultMessage: "Your local instance is ahead of the users you've shared it with.",
	},
	sharedInstancePublishButton: {
		id: 'app.instance.admonitions.shared-instance.publish-button',
		defaultMessage: 'Push update',
	},
	sharedInstancePublishingButton: {
		id: 'app.instance.admonitions.shared-instance.publishing-button',
		defaultMessage: 'Pushing...',
	},
	sharedInstanceReviewingButton: {
		id: 'app.instance.admonitions.shared-instance.reviewing-button',
		defaultMessage: 'Reviewing...',
	},
	sharedInstanceReviewHeader: {
		id: 'app.instance.admonitions.shared-instance.review-header',
		defaultMessage: 'Review changes',
	},
	sharedInstanceReviewAdmonitionHeader: {
		id: 'app.instance.admonitions.shared-instance.review-admonition-header',
		defaultMessage: 'Push update',
	},
	sharedInstanceReviewDescription: {
		id: 'app.instance.admonitions.shared-instance.review-description',
		defaultMessage: 'Review the content changes that will be shared with everyone using this instance.',
	},
	sharedInstanceAddedLabel: {
		id: 'app.instance.admonitions.shared-instance.added-label',
		defaultMessage: 'Added',
	},
	sharedInstanceRemovedLabel: {
		id: 'app.instance.admonitions.shared-instance.removed-label',
		defaultMessage: 'Removed',
	},
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const isPublishing = ref(false)
const isReviewingPublish = ref(false)
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)

const showSharedInstancePublishAdmonition = computed(
	() =>
		props.instance.shared_instance?.role === 'owner' &&
		props.instance.shared_instance.status === 'stale',
)

const stackItems = computed<InstanceAdmonitionItem[]>(() => {
	if (!showSharedInstancePublishAdmonition.value) return []

	return [
		{
			id: 'shared-instance-stale',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-stale',
		},
	]
})

async function reviewSharedInstanceChanges(e?: MouseEvent) {
	if (isPublishButtonDisabled.value) return

	isReviewingPublish.value = true
	try {
		const preview = await get_shared_instance_publish_preview(props.instance.id)
		if (!preview) return

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
		handleError(err as Error)
	} finally {
		isReviewingPublish.value = false
	}
}

async function publishSharedInstanceChanges() {
	if (isPublishing.value) return

	isPublishing.value = true
	try {
		await publish_shared_instance(props.instance.id)
		emit('published')
	} catch (err) {
		handleError(err as Error)
	} finally {
		isPublishing.value = false
	}
}
</script>
