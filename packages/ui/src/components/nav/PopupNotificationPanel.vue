<template>
	<div
		class="popup-notification-group"
		:class="{
			'has-sidebar': hasSidebar,
			'has-modal': hasModalActive && !hasModalOverride,
		}"
		:style="notificationGroupStyle"
	>
		<transition-group name="popup-notifs">
			<div
				v-for="item in notifications"
				:key="item.id"
				class="popup-notification-wrapper"
				:class="{ 'hidden-by-modal': hasModalActive && !item.showOverModal }"
				@mouseenter="stopTimer(item)"
				@mouseleave="setNotificationTimer(item)"
			>
				<NotificationToast
					v-if="item.toast"
					:type="item.toast.type"
					:actor-name="item.toast.actorName"
					:actor-avatar-url="item.toast.actorAvatarUrl"
					:entity-name="item.toast.entityName"
					:entity-icon-url="item.toast.entityIconUrl"
					:status-text="item.toast.statusText"
					:progress="item.toast.progress"
					:waiting="item.toast.waiting"
					:show-progress="item.toast.showProgress"
					:progress-type="item.toast.progressType"
					:progress-current="item.toast.progressCurrent"
					:progress-total="item.toast.progressTotal"
					@accept="handleToastAction(item, item.toast.onAccept)"
					@decline="handleToastAction(item, item.toast.onDecline)"
					@dismiss="handleToastAction(item, item.toast.onDismiss)"
					@launch="handleToastAction(item, item.toast.onLaunch)"
					@open-actor="item.toast.onOpenActor?.()"
					@open-instance="handleToastAction(item, item.toast.onOpenInstance)"
				/>
				<div v-else-if="isDownloadNotification(item)" class="flex flex-col gap-4">
					<div v-for="progressItem in downloadToastItems(item)" :key="progressItem.id">
						<NotificationToast
							type="instance-download"
							:entity-name="progressItem.title || item.title"
							:entity-icon-url="progressItem.iconUrl ?? item.iconUrl ?? MinecraftServerIcon"
							:status-text="progressItem.text"
							:progress="progressItem.progress"
							:waiting="progressItem.waiting"
							:show-progress="progressItem.showProgress"
							:wrap-text="progressItem.wrapText"
							:progress-type="progressItem.progressType"
							:progress-current="progressItem.progressCurrent"
							:progress-total="progressItem.progressTotal"
							:actions="progressItem.buttons"
							@dismiss="handleProgressItemDismiss(item, progressItem)"
							@action="(index) => handleProgressItemAction(progressItem, index)"
						/>
					</div>
				</div>
				<div
					v-else
					class="flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised shadow-xl border-surface-5 border-solid border p-4"
				>
					<div class="flex flex-col gap-2 w-full">
						<div class="flex items-center justify-between gap-2.5">
							<div class="flex min-w-0 flex-1 items-center gap-2">
								<component
									:is="item.titleLogo"
									v-if="item.titleLogo"
									class="h-7 w-auto min-w-0 max-w-full text-contrast"
								/>
								<template v-else>
									<div
										v-if="!item.hideIcon"
										class="flex items-center"
										:class="{
											'text-red': item.type === 'error',
											'text-orange': item.type === 'warning',
											'text-green': item.type === 'download',
											'text-contrast': item.type === 'success',
											'text-blue':
												!item.type ||
												!['error', 'warning', 'success', 'download'].includes(item.type),
										}"
									>
										<IssuesIcon v-if="item.type === 'warning'" class="h-5 w-5" />
										<DownloadIcon v-else-if="item.type === 'download'" class="h-5 w-5" />
										<CheckCircleIcon v-else-if="item.type === 'success'" class="h-5 w-5" />
										<XCircleIcon v-else-if="item.type === 'error'" class="h-5 w-5" />
										<InfoIcon v-else class="h-5 w-5" />
									</div>
									<div class="text-contrast font-semibold m-0 grow">
										{{ item.title }}
									</div>
								</template>
							</div>
							<ButtonStyled v-if="item.dismissible !== false" type="transparent" circular>
								<button class="-m-1.5" @click="dismiss(item.id)">
									<XIcon />
								</button>
							</ButtonStyled>
						</div>
						<span v-if="item.text" class="text-primary">
							{{ item.text }}
						</span>
						<component
							:is="item.bodyComponent"
							v-if="item.bodyComponent"
							v-bind="item.bodyProps ?? {}"
						/>
					</div>
					<div v-if="item.progressItems?.length" class="flex flex-col gap-3">
						<div
							v-for="progressItem in item.progressItems"
							:key="progressItem.id"
							class="flex flex-col gap-2"
						>
							<div class="text-contrast truncate">
								{{ progressItem.title }}
							</div>
							<ProgressBar
								:progress="progressItem.progress"
								:max="1"
								:waiting="progressItem.waiting"
								:color="progressColorForType(item.type)"
								:gradient-border="false"
								full-width
							/>
							<div v-if="progressItem.text" class="text-sm text-secondary truncate">
								{{ progressItem.text }}
							</div>
						</div>
					</div>
					<ProgressBar
						v-else-if="item.progress != null || item.waiting"
						:progress="item.progress ?? 0"
						:max="1"
						:waiting="item.waiting ?? false"
						:color="progressColorForType(item.type)"
						:gradient-border="false"
						full-width
					/>
					<div v-if="item.buttons?.length" class="flex gap-1.5">
						<ButtonStyled
							v-for="(btn, idx) in item.buttons"
							:key="idx"
							:color="btn.color || (idx === 0 ? 'brand' : undefined)"
						>
							<button @click="handleButtonClick(item.id, btn)" class="!shadow-none">
								<component :is="btn.icon" v-if="btn.icon" />
								{{ btn.label }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</transition-group>
	</div>
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	DownloadIcon,
	InfoIcon,
	IssuesIcon,
	MinecraftServerIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import { useModalStack } from '../../composables/modal-stack'
import {
	injectPopupNotificationManager,
	type PopupNotification,
	type PopupNotificationButton,
	type PopupNotificationProgressItem,
} from '../../providers'
import ButtonStyled from '../base/ButtonStyled.vue'
import ProgressBar from '../base/ProgressBar.vue'
import NotificationToast from '../notifications/NotificationToast.vue'

const popupNotificationManager = injectPopupNotificationManager()
const notifications = computed<PopupNotification[]>(() =>
	popupNotificationManager.getNotifications(),
)
const { stackCount } = useModalStack()
const hasModalActive = computed(() => stackCount.value > 0)
const hasModalOverride = computed(() => notifications.value.some((item) => item.showOverModal))
const notificationGroupStyle = computed(() => ({
	zIndex: hasModalActive.value ? 100 + stackCount.value * 10 + 8 : 200,
}))

const stopTimer = (n: PopupNotification) => popupNotificationManager.stopNotificationTimer(n)
const setNotificationTimer = (n: PopupNotification) =>
	popupNotificationManager.setNotificationTimer(n)
const dismiss = (id: string | number) => popupNotificationManager.removeNotification(id)

function isDownloadNotification(item: PopupNotification) {
	return (
		item.type === 'download' &&
		(!!item.progressItems?.length || item.progress != null || item.waiting)
	)
}

function downloadToastItems(item: PopupNotification): PopupNotificationProgressItem[] {
	if (item.progressItems?.length) {
		return item.progressItems
	}

	return [
		{
			id: `${item.id}`,
			title: item.title,
			text: item.text,
			iconUrl: item.iconUrl,
			progress: item.progress ?? 0,
			waiting: item.waiting ?? false,
			showProgress: true,
			progressType: 'percentage',
		},
	]
}

async function handleProgressItemDismiss(
	item: PopupNotification,
	progressItem: PopupNotificationProgressItem,
) {
	if (progressItem.onDismiss) {
		await progressItem.onDismiss()
		return
	}

	dismiss(item.id)
}

async function handleProgressItemAction(
	progressItem: PopupNotificationProgressItem,
	index: number,
) {
	const button = progressItem.buttons?.[index]
	if (button) {
		await handleProgressItemButtonClick(progressItem, button)
	}
}

async function handleProgressItemButtonClick(
	progressItem: PopupNotificationProgressItem,
	btn: PopupNotificationButton,
) {
	await btn.action()
	if (!btn.keepOpen) {
		await progressItem.onDismiss?.()
	}
}

async function handleButtonClick(id: string | number, btn: PopupNotificationButton) {
	await btn.action()
	if (!btn.keepOpen) {
		popupNotificationManager.removeNotification(id)
	}
}

async function handleToastAction(item: PopupNotification, action?: () => void | Promise<void>) {
	popupNotificationManager.removeNotification(item.id)
	await action?.()
}

function progressColorForType(type: PopupNotification['type']) {
	if (type === 'error') {
		return 'red'
	} else if (type === 'warning') {
		return 'orange'
	} else if (type === 'download') {
		return 'green'
	} else if (type === 'success') {
		return 'green'
	} else if (type === 'info') {
		return 'blue'
	}
	return 'green'
}

withDefaults(
	defineProps<{
		hasSidebar?: boolean
	}>(),
	{
		hasSidebar: false,
	},
)
</script>

<style scoped>
.popup-notification-group {
	position: fixed;
	top: calc(var(--top-bar-height, 3rem) + 1.5rem);
	right: 1.5rem;
	z-index: 200;
	width: min(440px, calc(100vw - 1.5rem));
	min-width: min(440px, calc(100vw - 1.5rem));
	max-width: min(440px, calc(100vw - 1.5rem));
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	transition:
		opacity 0.2s ease-in-out,
		transform 0.2s ease-in-out;
}

.popup-notification-group.has-sidebar {
	right: calc(var(--right-bar-width, 0px) + 1.5rem);
}

.popup-notification-group.has-modal {
	opacity: 0;
	pointer-events: none;
	transform: translateY(-0.5rem);
}

@media screen and (max-width: 500px) {
	.popup-notification-group {
		right: 0.75rem;
	}
}

.popup-notification-group .popup-notification-wrapper {
	width: 100%;
}

.popup-notification-group .popup-notification-wrapper.hidden-by-modal {
	display: none;
}

.popup-notifs-enter-active,
.popup-notifs-leave-active,
.popup-notifs-move {
	transition: all 0.3s ease-in-out;
}

.popup-notifs-enter-from {
	opacity: 0;
	transform: translateX(100%) scale(0.8);
}

.popup-notifs-leave-to {
	opacity: 0;
	transform: translateX(100%) scale(0.8);
}
</style>
