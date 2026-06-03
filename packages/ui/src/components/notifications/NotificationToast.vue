<template>
	<div
		class="notification-toast relative overflow-hidden rounded-[20px] border border-solid border-surface-5 bg-surface-3 p-4 shadow-[0px_4px_8px_0px_rgba(0,0,0,0.1),0px_1px_3px_0px_rgba(0,0,0,0.2)]"
	>
		<div v-if="isInviteNotification" class="flex w-full items-start gap-3">
			<Avatar
				:src="actorAvatarUrl"
				:alt="actorLabel"
				:tint-by="actorLabel"
				size="44px"
				circle
				no-shadow
				class="border border-solid border-surface-5"
			/>
			<div class="flex min-w-0 flex-1 flex-col gap-2.5">
				<div class="flex w-full items-start gap-1">
					<p class="m-0 min-w-0 flex-1 break-words text-lg font-normal leading-6 text-contrast/85">
						<template v-if="type === 'friend-request'">
							<span class="font-semibold text-contrast">{{ actorLabel }}</span>
							<span> sent you a friend request.</span>
						</template>
						<template v-else>
							<button
								v-if="actorName"
								type="button"
								class="m-0 inline border-0 bg-transparent p-0 text-lg font-semibold leading-6 text-contrast hover:underline"
								@click="$emit('open-actor')"
							>
								{{ actorName }}
							</button>
							<span v-else class="font-semibold text-contrast">Someone</span>
							<span class="mx-1">{{ inviteActionText }}</span>
							<template v-if="type === 'server-invite'">
								<span class="font-semibold text-contrast">{{ entityLabel }}</span
								>.
							</template>
							<template v-else>
								<span class="inline-flex max-w-full items-center gap-[5px] align-[-4px]">
									<Avatar
										:src="entityIconUrl"
										:alt="entityLabel"
										size="24px"
										no-shadow
										raised
										:tint-by="entityLabel"
										class="!rounded-[7px]"
									/>
									<span class="min-w-0 truncate font-semibold text-contrast">{{
										entityLabel
									}}</span> </span
								>.
							</template>
						</template>
					</p>
					<ButtonStyled size="small" type="transparent" circular>
						<button
							type="button"
							class="notification-toast-dismiss"
							aria-label="Dismiss notification"
							@click="$emit('dismiss')"
						>
							<XIcon />
						</button>
					</ButtonStyled>
				</div>
				<div class="flex items-center gap-2">
					<ButtonStyled color="brand">
						<button @click="$emit('accept')">Accept</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button @click="$emit('decline')">Decline</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div v-else class="flex w-full items-start gap-3">
			<Avatar
				:src="entityIconUrl"
				:alt="entityLabel"
				size="44px"
				no-shadow
				raised
				:tint-by="entityLabel"
				class="!rounded-xl border border-solid border-surface-5"
			/>
			<div class="flex min-w-0 flex-1 flex-col" :class="{ 'gap-2.5': type === 'instance-ready' }">
				<div class="flex min-w-0 flex-1 items-start gap-1">
					<div class="flex min-w-0 flex-1 flex-col gap-[3px] text-base leading-5">
						<p
							ref="titleRef"
							v-tooltip="truncatedTooltip(titleRef, entityLabel)"
							class="m-0 min-w-0 truncate text-lg font-semibold leading-6 text-contrast"
						>
							{{ entityLabel }}
						</p>
						<p
							ref="statusRef"
							v-tooltip="truncatedTooltip(statusRef, statusLine)"
							class="m-0 min-w-0 truncate font-normal leading-tight text-contrast/85"
						>
							{{ statusLine }}
						</p>
					</div>
					<ButtonStyled size="small" type="transparent" circular>
						<button
							type="button"
							class="notification-toast-dismiss"
							aria-label="Dismiss notification"
							@click="$emit('dismiss')"
						>
							<XIcon />
						</button>
					</ButtonStyled>
				</div>
				<div v-if="type === 'instance-ready'" class="flex items-center gap-2">
					<ButtonStyled color="brand">
						<button @click="$emit('launch')">Launch game</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button @click="$emit('open-instance')">Instance</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div
			v-if="showsBottomProgress"
			class="notification-bottom-progress-track absolute inset-x-0 bottom-0 h-1 overflow-hidden"
			role="progressbar"
			:aria-valuenow="isWaitingProgress ? undefined : progressPercent"
			aria-valuemin="0"
			aria-valuemax="100"
		>
			<div
				class="h-full transition-[left,width] duration-200 ease-in-out"
				:class="[
					type === 'instance-ready' ? 'bg-surface-5' : 'bg-brand',
					{ 'notification-bottom-progress--waiting': isWaitingProgress },
				]"
				:style="isWaitingProgress ? undefined : { width: `${progressPercent}%` }"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { truncatedTooltip } from '../../utils/truncate'
import Avatar from '../base/Avatar.vue'
import ButtonStyled from '../base/ButtonStyled.vue'

type NotificationToastType =
	| 'friend-request'
	| 'server-invite'
	| 'instance-invite'
	| 'instance-download'
	| 'instance-ready'

const props = withDefaults(
	defineProps<{
		type: NotificationToastType
		actorName?: string | null
		actorAvatarUrl?: string | null
		entityName?: string
		entityIconUrl?: string | null
		statusText?: string
		progress?: number
		waiting?: boolean
	}>(),
	{
		actorName: null,
		actorAvatarUrl: null,
		entityName: '',
		entityIconUrl: null,
		waiting: false,
	},
)

defineEmits<{
	accept: []
	decline: []
	dismiss: []
	launch: []
	'open-actor': []
	'open-instance': []
}>()

const isInviteNotification = computed(
	() =>
		props.type === 'friend-request' ||
		props.type === 'server-invite' ||
		props.type === 'instance-invite',
)

const actorLabel = computed(() => props.actorName || 'Someone')
const entityLabel = computed(() => props.entityName || '')
const progressValue = computed(() => Math.max(0, Math.min(1, props.progress ?? 0)))
const progressPercent = computed(() => Math.round(progressValue.value * 100))
const isWaitingProgress = computed(() => props.type === 'instance-download' && props.waiting)

const inviteActionText = computed(() => {
	if (props.type === 'server-invite') {
		return 'invited you to manage the server'
	}

	return 'invited you to play the instance'
})

const resolvedStatusText = computed(() => {
	if (props.type === 'instance-ready') {
		return props.statusText ?? 'Installed and ready to play.'
	}

	return props.statusText ?? ''
})

const statusLine = computed(() => {
	if (props.type !== 'instance-download' || props.waiting) {
		return resolvedStatusText.value
	}

	const status = resolvedStatusText.value.trim()
	return status ? `${status} ${progressPercent.value}%` : `${progressPercent.value}%`
})

const showsBottomProgress = computed(
	() =>
		props.type === 'instance-download' ||
		(props.type === 'instance-ready' && props.progress != null),
)

const titleRef = ref<HTMLElement | null>(null)
const statusRef = ref<HTMLElement | null>(null)
</script>

<style scoped>
.notification-toast {
	width: min(420px, calc(100vw - 1.5rem));
}

.notification-toast-dismiss {
	--_height: 1.25rem;
	--_width: 1.25rem;
	--_padding-x: 0;
	--_padding-y: 0;
	--_icon-size: 1.25rem;
	--_box-shadow: none;
	--_text: var(--color-base);
	--_hover-bg: transparent;
	--_hover-text: var(--color-contrast);
}

.notification-bottom-progress--waiting {
	animation: notification-bottom-progress-waiting 1s linear infinite;
	position: relative;
	width: 20%;
}

@keyframes notification-bottom-progress-waiting {
	0% {
		left: -20%;
	}

	100% {
		left: 100%;
	}
}

.notification-bottom-progress-track {
	background-color: color-mix(in srgb, var(--surface-2) 50%, transparent);
}
</style>
