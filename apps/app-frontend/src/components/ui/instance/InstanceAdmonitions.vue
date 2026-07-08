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
						<button
							class="!h-10"
							:disabled="isPublishButtonDisabled"
							@click="reviewSharedInstanceChanges"
						>
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
			<Admonition
				v-else-if="item.kind === 'shared-instance-wrong-account'"
				type="warning"
				:header="formatMessage(sharedInstanceWrongAccountHeader)"
			>
				{{
					formatMessage(sharedInstanceWrongAccountBody, {
						username: sharedInstanceExpectedUsername,
					})
				}}
			</Admonition>
			<Admonition
				v-else-if="item.kind === 'shared-instance-unavailable'"
				type="warning"
				:header="formatMessage(messages.sharedInstanceUnavailableTitle)"
			>
				{{
					formatMessage(sharedInstanceUnavailableTextMessage(sharedInstanceUnavailableReason), {
						manager: sharedInstanceUnavailableManager,
					})
				}}
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

import {
	getErrorMessage,
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import { get_shared_instance_publish_preview, publish_shared_instance } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

defineOptions({
	inheritAttrs: false,
})

type InstanceAdmonitionItem = StackedAdmonitionItem & {
	kind:
		| 'shared-instance-stale'
		| 'shared-instance-unavailable'
		| 'shared-instance-wrong-account'
}

const props = defineProps<{
	instance: GameInstance
	sharedInstanceUnavailableReason?: SharedInstanceUnavailableReason | null
	sharedInstanceUnavailableManager?: string | null
	sharedInstanceWrongAccount?: boolean
	sharedInstanceExpectedUsername?: string | null
	sharedInstanceRole?: 'owner' | 'member' | null
	sharedInstanceSignedOut?: boolean
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
		defaultMessage:
			'Review the content changes that will be shared with everyone using this instance.',
	},
	sharedInstanceAddedLabel: {
		id: 'app.instance.admonitions.shared-instance.added-label',
		defaultMessage: 'Added',
	},
	sharedInstanceRemovedLabel: {
		id: 'app.instance.admonitions.shared-instance.removed-label',
		defaultMessage: 'Removed',
	},
	sharedInstanceUnavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
	},
	sharedInstanceUnavailableText: {
		id: 'instance.shared-instance.unavailable.text',
		defaultMessage:
			'The shared instance has been deleted or your access has been revoked. Contact {manager} for more information.',
	},
	sharedInstanceDeletedText: {
		id: 'instance.shared-instance.unavailable.deleted-text',
		defaultMessage: 'The shared instance has been deleted. Contact {manager} for more information.',
	},
	sharedInstanceAccessRevokedText: {
		id: 'instance.shared-instance.unavailable.access-revoked-text',
		defaultMessage:
			'Your access to this shared instance has been revoked. Contact {manager} for more information.',
	},
	sharedInstanceUnavailableFallbackManager: {
		id: 'instance.shared-instance.unavailable.manager-fallback',
		defaultMessage: 'the instance manager',
	},
	sharedInstanceErrorTitle: {
		id: 'instance.shared-instance.error.title',
		defaultMessage: 'Something has gone wrong',
	},
	sharedInstanceWrongAccountHeader: {
		id: 'app.instance.shared-instance-wrong-account.warning-header',
		defaultMessage: 'You are using the wrong Modrinth account',
	},
	sharedInstanceSignedOutHeader: {
		id: 'app.instance.shared-instance-wrong-account.signed-out-warning-header',
		defaultMessage: 'Sign in to the correct Modrinth account first',
	},
	sharedInstanceWrongAccountUserBody: {
		id: 'app.instance.shared-instance-wrong-account.user-admonition-body',
		defaultMessage:
			'Sign in as {username} to receive updates for this shared instance. Shared instance functionality is disabled until you use the correct account.',
	},
	sharedInstanceWrongAccountOwnerBody: {
		id: 'app.instance.shared-instance-wrong-account.owner-admonition-body',
		defaultMessage:
			'Sign in as {username} to manage sharing and publish updates for this shared instance. Shared instance functionality is disabled until you use the correct account.',
	},
	sharedInstanceWrongAccountFallbackUsername: {
		id: 'app.instance.shared-instance-wrong-account.fallback-username',
		defaultMessage: 'the linked account',
	},
})

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const isPublishing = ref(false)
const isReviewingPublish = ref(false)
const publishReviewModal = ref<InstanceType<typeof ContentDiffModal>>()
const publishDiffs = ref<ContentDiffItem[]>([])

const isPublishButtonDisabled = computed(() => isPublishing.value || isReviewingPublish.value)
const sharedInstanceUnavailableReason = computed(
	() => props.sharedInstanceUnavailableReason ?? null,
)
const sharedInstanceUnavailableManager = computed(
	() =>
		props.sharedInstanceUnavailableManager ||
		formatMessage(messages.sharedInstanceUnavailableFallbackManager),
)
const sharedInstanceExpectedUsername = computed(
	() =>
		props.sharedInstanceExpectedUsername ||
		formatMessage(messages.sharedInstanceWrongAccountFallbackUsername),
)
const sharedInstanceWrongAccount = computed(() => props.sharedInstanceWrongAccount ?? false)
const sharedInstanceWrongAccountHeader = computed(() =>
	props.sharedInstanceSignedOut
		? messages.sharedInstanceSignedOutHeader
		: messages.sharedInstanceWrongAccountHeader,
)
const sharedInstanceWrongAccountBody = computed(() =>
	props.sharedInstanceRole === 'owner'
		? messages.sharedInstanceWrongAccountOwnerBody
		: messages.sharedInstanceWrongAccountUserBody,
)

const showSharedInstancePublishAdmonition = computed(
	() =>
		!sharedInstanceWrongAccount.value &&
		props.instance.shared_instance?.role === 'owner' &&
		props.instance.shared_instance.status === 'stale',
)

const stackItems = computed<InstanceAdmonitionItem[]>(() => {
	const items: InstanceAdmonitionItem[] = []

	if (sharedInstanceWrongAccount.value) {
		items.push({
			id: 'shared-instance-wrong-account',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-wrong-account',
		})
	}

	if (sharedInstanceUnavailableReason.value) {
		items.push({
			id: 'shared-instance-unavailable',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-unavailable',
		})
	}

	if (showSharedInstancePublishAdmonition.value) {
		items.push({
			id: 'shared-instance-stale',
			type: 'warning',
			dismissible: false,
			kind: 'shared-instance-stale',
		})
	}

	return items
})

function sharedInstanceUnavailableTextMessage(reason: SharedInstanceUnavailableReason | null) {
	if (reason === 'deleted') return messages.sharedInstanceDeletedText
	if (reason === 'access_revoked') return messages.sharedInstanceAccessRevokedText
	return messages.sharedInstanceUnavailableText
}

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

async function reviewSharedInstanceChanges(e?: MouseEvent) {
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

async function publishSharedInstanceChanges() {
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
