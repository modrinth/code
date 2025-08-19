<template>
	<div
		class="medal-promotion relative flex w-full flex-row items-center justify-between rounded-2xl p-4 shadow-xl"
	>
		<MedalBackgroundImage />

		<div class="z-10 mr-2 flex flex-col gap-1">
			<Transition
				enter-from-class="opacity-0 translate-y-1"
				enter-active-class="transition-all duration-300"
				enter-to-class="opacity-100 translate-y-0"
				leave-from-class="opacity-100 translate-y-0"
				leave-active-class="transition-all duration-150"
				leave-to-class="opacity-0 -translate-y-1"
			>
				<div
					v-if="expiryDate"
					class="flex items-center gap-2 whitespace-nowrap font-semibold text-contrast"
				>
					<ClockIcon class="clock-glow text-medal-orange size-5 shrink-0" />
					<span class="w-full text-wrap text-lg">
						Your <span class="text-medal-orange">Medal</span>-powered Modrinth Server will expire in
						<span class="text-medal-orange font-bold">{{ timeLeftCountdown.days }}</span> days
						<span class="text-medal-orange font-bold">{{ timeLeftCountdown.hours }}</span> hours
						<span class="text-medal-orange font-bold">{{ timeLeftCountdown.minutes }}</span> minutes
						<span class="text-medal-orange font-bold">{{ timeLeftCountdown.seconds }}</span>
						seconds.
					</span>
				</div>
			</Transition>
		</div>

		<ButtonStyled color="medal-promo" type="outlined" size="large">
			<button class="z-10 my-auto" @click="openUpgradeModal"><RocketIcon /> Upgrade</button>
		</ButtonStyled>
	</div>
	<ServersUpgradeModalWrapper ref="upgradeModal" />
</template>

<script setup lang="ts">
import { ClockIcon, RocketIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import type { UserSubscription } from '@modrinth/utils'
import dayjs from 'dayjs'
import dayjsDuration from 'dayjs/plugin/duration'
import type { ComponentPublicInstance } from 'vue'

import MedalBackgroundImage from '~/components/ui/servers/marketing/MedalBackgroundImage.vue'

import ServersUpgradeModalWrapper from '../ServersUpgradeModalWrapper.vue'

dayjs.extend(dayjsDuration)

type UpgradeWrapperRef = ComponentPublicInstance<{ open: (id?: string) => void | Promise<void> }>
const upgradeModal = ref<UpgradeWrapperRef | null>(null)

const props = defineProps<{
	serverId?: string
}>()

const { data: subscriptions } = await useLazyAsyncData(
	'countdown-subscriptions',
	() =>
		useBaseFetch(`billing/subscriptions`, {
			internal: true,
		}) as Promise<UserSubscription[]>,
)

const expiryDate = computed(() => {
	for (const subscription of subscriptions.value || []) {
		if (subscription.metadata?.id === props.serverId) {
			return dayjs(subscription.created).add(5, 'days')
		}
	}

	return undefined
})

function openUpgradeModal() {
	upgradeModal.value?.open(props.serverId)
}

const timeLeftCountdown = ref({ days: 0, hours: 0, minutes: 0, seconds: 0 })

function updateCountdown() {
	if (!expiryDate.value) {
		timeLeftCountdown.value = { days: 0, hours: 0, minutes: 0, seconds: 0 }
		return
	}

	const now = dayjs()
	const diff = expiryDate.value.diff(now)

	if (diff <= 0) {
		timeLeftCountdown.value = { days: 0, hours: 0, minutes: 0, seconds: 0 }
		return
	}

	const duration = dayjs.duration(diff)
	timeLeftCountdown.value = {
		days: duration.days(),
		hours: duration.hours(),
		minutes: duration.minutes(),
		seconds: duration.seconds(),
	}
}

updateCountdown()

const intervalId = ref<NodeJS.Timeout | null>(null)
onMounted(() => {
	intervalId.value = setInterval(updateCountdown, 1000)
})

onUnmounted(() => {
	if (intervalId.value) clearInterval(intervalId.value)
})
</script>

<style scoped lang="scss">
.medal-promotion {
	position: relative;
	border: 1px solid var(--medal-promotion-bg-orange);
	background: inherit; // allows overlay + pattern to take over
	overflow: hidden;
}

.overlay {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background: var(--medal-promotion-bg-gradient);
	z-index: 1;
	border-radius: inherit;
}

.background-pattern {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	z-index: 0;
	background-color: var(--medal-promotion-bg);
	border-radius: inherit;
	color: var(--medal-promotion-text-orange);
}

.clock-glow {
	filter: drop-shadow(0 0 72px var(--color-orange)) drop-shadow(0 0 36px var(--color-orange))
		drop-shadow(0 0 18px var(--color-orange));
}

.text-medal-orange {
	color: var(--medal-promotion-text-orange);
	font-weight: bold;
}
</style>
