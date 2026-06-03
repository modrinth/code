<template>
	<div
		class="popup-notification-group"
		:class="{
			'has-sidebar': hasSidebar,
		}"
	>
		<transition-group name="popup-notifs">
			<div
				v-for="item in notifications"
				:key="item.id"
				class="popup-notification-wrapper"
				@mouseenter="stopTimer(item)"
				@mouseleave="setNotificationTimer(item)"
			>
				<div v-if="isDownloadNotification(item)" class="flex flex-col gap-4">
					<NotificationToast
						v-for="progressItem in downloadToastItems(item)"
						:key="progressItem.id"
						type="instance-download"
						:entity-name="progressItem.title || item.title"
						:entity-icon-url="MinecraftServerIcon"
						:status-text="downloadStatusText(progressItem)"
						:progress="progressItem.progress"
						:waiting="progressItem.waiting"
						@dismiss="dismiss(item.id)"
					/>
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
							<ButtonStyled size="small" type="transparent" circular>
								<button @click="dismiss(item.id)">
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
							<button @click="handleButtonClick(item.id, btn)">
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
			progress: item.progress ?? 0,
			waiting: item.waiting ?? false,
		},
	]
}

function downloadStatusText(progressItem: PopupNotificationProgressItem) {
	return progressItem.text?.replace(/^\d+%\s*/, '') ?? ''
}

function handleButtonClick(id: string | number, btn: PopupNotificationButton) {
	btn.action()
	if (!btn.keepOpen) {
		popupNotificationManager.removeNotification(id)
	}
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
	width: 360px;
	min-width: 360px;
	max-width: 360px;
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.popup-notification-group.has-sidebar {
	right: calc(var(--right-bar-width, 0px) + 1.5rem);
}

@media screen and (max-width: 500px) {
	.popup-notification-group {
		right: 0.75rem;
	}
}

.popup-notification-group .popup-notification-wrapper {
	width: 100%;
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
