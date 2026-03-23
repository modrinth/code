<template>
	<div
		class="popup-notification-group experimental-styles-within"
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
				<div
					class="flex w-full flex-col gap-3 overflow-hidden rounded-2xl bg-bg-raised shadow-xl border-surface-5 border-solid border p-4"
				>
					<div class="flex flex-col gap-2 w-full">
						<div class="flex items-center justify-between gap-2.5">
							<div class="flex items-center gap-2">
								<div
									class="flex items-center"
									:class="{
										'text-red': item.type === 'error',
										'text-orange': item.type === 'warning',
										'text-contrast': item.type === 'success',
										'text-blue': !item.type || !['error', 'warning', 'success'].includes(item.type),
									}"
								>
									<IssuesIcon v-if="item.type === 'warning'" class="h-5 w-5" />
									<CheckCircleIcon v-else-if="item.type === 'success'" class="h-5 w-5" />
									<XCircleIcon v-else-if="item.type === 'error'" class="h-5 w-5" />
									<InfoIcon v-else class="h-5 w-5" />
								</div>
								<div class="text-contrast font-semibold m-0 grow">
									{{ item.title }}
								</div>
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
					</div>
					<div v-if="item.buttons?.length" class="flex gap-1.5">
						<ButtonStyled
							v-for="(btn, idx) in item.buttons"
							:key="idx"
							:color="btn.color || (idx === 0 ? 'brand' : undefined)"
						>
							<button @click="handleButtonClick(item.id, btn)">
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
import { CheckCircleIcon, InfoIcon, IssuesIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import { computed } from 'vue'

import {
	injectPopupNotificationManager,
	type PopupNotification,
	type PopupNotificationButton,
} from '../../providers'
import ButtonStyled from '../base/ButtonStyled.vue'

const popupNotificationManager = injectPopupNotificationManager()
const notifications = computed<PopupNotification[]>(() =>
	popupNotificationManager.getNotifications(),
)

const stopTimer = (n: PopupNotification) => popupNotificationManager.stopNotificationTimer(n)
const setNotificationTimer = (n: PopupNotification) =>
	popupNotificationManager.setNotificationTimer(n)
const dismiss = (id: string | number) => popupNotificationManager.removeNotification(id)

function handleButtonClick(id: string | number, btn: PopupNotificationButton) {
	btn.action()
	popupNotificationManager.removeNotification(id)
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
	width: 400px;
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.popup-notification-group.has-sidebar {
	right: calc(var(--right-bar-width, 0px) + 1.5rem);
}

@media screen and (max-width: 500px) {
	.popup-notification-group {
		width: calc(100% - 1.5rem);
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
