<script setup lang="ts">
import { ModrinthApiError } from '@modrinth/api-client'
import { EyeIcon, TrashIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	type ButtonMenuOption,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectPopupNotificationManager,
	NewModal,
	SplitButton,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils/parse'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { injectInstanceActions } from '@/features/instances/instance-actions/context'
import { toError } from '@/helpers/errors'
import { getInstanceIconUrl, list } from '@/helpers/instance'
import { sharedInstanceErrorMessages } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'
import { refreshSharedInstanceLock } from '@/platform/app-lib/shared-instances/commands'

type LockNotification = {
	id: string | number
	body?: {
		type?: unknown
		shared_instance_id?: unknown
		shared_instance_name?: unknown
		shared_instance_icon?: unknown
		reason?: unknown
	}
}

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const router = useRouter()
const { handleError } = injectNotificationManager()
const popupNotificationManager = injectPopupNotificationManager()
const instanceActions = injectInstanceActions()

const modal = ref<InstanceType<typeof NewModal>>()
const reasonScroll = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade } = useScrollIndicator(reasonScroll)

const instance = ref<GameInstance | null>(null)
const instanceName = ref('')
const iconUrl = ref<string | null>(null)
const reason = ref<string | null>(null)
const renderedReason = computed(() => (reason.value ? renderString(reason.value) : ''))

const viewOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'delete-instance',
		label: formatMessage(messages.deleteInstance),
		icon: TrashIcon,
		tone: 'red',
		action: () => {
			if (instance.value) deleteInstance(instance.value)
		},
	},
])

const displayedNotifications = new Set<string | number>()
const popupNotificationIds = new Set<string | number>()

async function markRead(notification: LockNotification) {
	try {
		await client.labrinth.notifications_v2.markAsRead(String(notification.id))
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) return
		handleError(toError(error))
	}
}

function openInstance(target: GameInstance) {
	modal.value?.hide()
	void router.push(`/instance/${encodeURIComponent(target.id)}`)
}

function deleteInstance(target: GameInstance) {
	modal.value?.hide()
	instanceActions.confirmDelete(target)
}

async function handleNotification(notification: LockNotification) {
	const body = notification.body
	if (body?.type !== 'shared_instance_quarantined') return false
	if (displayedNotifications.has(notification.id)) return true
	displayedNotifications.add(notification.id)

	const instances = await list().catch(() => [])
	if (!displayedNotifications.has(notification.id)) return true

	const localInstance = instances.find(
		(candidate) => candidate.shared_instance?.id === body.shared_instance_id,
	)
	if (localInstance) void refreshSharedInstanceLock(localInstance.id).catch(() => {})

	const name = typeof body.shared_instance_name === 'string' ? body.shared_instance_name : ''
	const entityIconUrl =
		getInstanceIconUrl(localInstance?.icon_path) ??
		(typeof body.shared_instance_icon === 'string' ? body.shared_instance_icon : null)
	const toast = { contentType: 'toast' as const, title: name, entityName: name, entityIconUrl }
	const onDismiss = () => markRead(notification)

	const popupNotification =
		localInstance?.shared_instance?.role === 'member'
			? popupNotificationManager.addPopupNotification({
					...toast,
					type: 'instance-quarantined-member',
					autoCloseMs: null,
					onDismiss,
					onOpenInstance: () => {
						void markRead(notification)
						openInstance(localInstance)
					},
					onDelete: () => {
						void markRead(notification)
						deleteInstance(localInstance)
					},
				})
			: popupNotificationManager.addPopupNotification({
					...toast,
					type: 'instance-quarantined',
					autoCloseMs: null,
					onDismiss,
					onReview: () => {
						popupNotificationManager.removeNotification(popupNotification.id)
						void markRead(notification)
						instance.value = localInstance ?? null
						instanceName.value = name
						iconUrl.value = entityIconUrl
						reason.value = typeof body.reason === 'string' && body.reason ? body.reason : null
						modal.value?.show()
					},
				})
	popupNotificationIds.add(popupNotification.id)

	return true
}

function clearNotifications() {
	for (const id of popupNotificationIds) {
		popupNotificationManager.removeNotification(id)
	}
	popupNotificationIds.clear()
	displayedNotifications.clear()
}

defineExpose({ handleNotification, clearNotifications })

const messages = defineMessages({
	title: {
		id: 'app.shared-instance.lock-notice.title',
		defaultMessage: 'Shared instance locked',
	},
	reason: {
		id: 'app.shared-instance.lock-notice.reason',
		defaultMessage: 'Reason',
	},
	deleteInstance: {
		id: 'app.shared-instance.lock-notice.delete-instance',
		defaultMessage: 'Delete instance',
	},
	moreActions: {
		id: 'app.shared-instance.lock-notice.more-actions',
		defaultMessage: 'More instance actions',
	},
})
</script>

<template>
	<NewModal ref="modal" max-width="544px" width="544px">
		<template #title>
			<Avatar
				:src="iconUrl"
				:alt="instanceName"
				:tint-by="instanceName"
				size="3rem"
				no-shadow
				raised
				class="!rounded-xl border border-solid border-surface-5"
			/>
			<span class="text-2xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</span>
		</template>

		<div class="flex flex-col gap-4">
			<Admonition
				type="warning"
				:header="formatMessage(sharedInstanceErrorMessages.lockedOwnerTitle)"
				:body="formatMessage(sharedInstanceErrorMessages.lockedOwnerText)"
			/>
			<div v-if="reason" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.reason) }}</span>
				<div
					class="relative overflow-hidden rounded-xl border border-solid border-surface-4 bg-surface-2"
				>
					<!-- eslint-disable-next-line vue/no-v-html -->
					<div
						ref="reasonScroll"
						class="markdown-body max-h-64 overflow-y-auto break-words px-4 py-3 text-primary"
						v-html="renderedReason"
					/>
					<div
						v-if="showTopFade"
						aria-hidden="true"
						class="pointer-events-none absolute inset-x-0 top-0 h-6 bg-gradient-to-b from-surface-2 to-transparent"
					/>
					<div
						v-if="showBottomFade"
						aria-hidden="true"
						class="pointer-events-none absolute inset-x-0 bottom-0 h-6 bg-gradient-to-t from-surface-2 to-transparent"
					/>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.closeButton) }}
				</Button>
				<SplitButton
					v-if="instance"
					type="colored"
					color="brand"
					:menu-label="formatMessage(messages.moreActions)"
					:options="viewOptions"
					@click="openInstance(instance)"
				>
					<EyeIcon />
					{{ formatMessage(commonMessages.viewLabel) }}
				</SplitButton>
			</div>
		</template>
	</NewModal>
</template>
