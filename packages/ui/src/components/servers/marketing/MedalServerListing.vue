<template>
	<div
		class="transition-all"
		:class="{
			pressable: !isDisabled,
			hoverable: !isDisabled,
			'cursor-pointer': !isDisabled,
		}"
		:role="!isDisabled ? 'link' : undefined"
		:tabindex="!isDisabled ? 0 : undefined"
		@click="navigateToServer"
		@keydown.enter.self="navigateToServer"
		@keydown.space.prevent.self="navigateToServer"
	>
		<div
			class="medal-promotion flex flex-row items-center overflow-x-hidden rounded-2xl p-4 transition-all duration-150"
			:class="{
				'!rounded-b-none border-b-0': hasNotice,
				'!bg-surface-2': isDisabled,
			}"
			data-pyro-server-listing
			:data-pyro-server-listing-id="server_id"
		>
			<MedalBackgroundImage />
			<div
				v-if="isDisabled"
				class="relative z-10 flex size-16 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
			>
				<Avatar src="https://cdn-raw.modrinth.com/medal_icon.webp" size="64px" class="opacity-50" />
				<SpinnerIcon
					v-if="isUpgrading"
					class="size-8 animate-spin absolute text-contrast"
					:class="{ 'opacity-50': isDisabled }"
				/>
				<LockIcon v-else class="size-8 absolute" :class="{ 'opacity-50': isDisabled }" />
			</div>
			<Avatar v-else src="https://cdn-raw.modrinth.com/medal_icon.webp" size="64px" class="z-10" />
			<div class="z-10 ml-4 flex min-w-0 flex-col gap-1.5">
				<div class="flex flex-row items-center gap-2">
					<h2
						class="m-0 truncate text-xl font-bold text-contrast"
						:class="{ 'opacity-50': isDisabled }"
					>
						{{ name }}
					</h2>

					<span class="truncate" :class="{ 'opacity-50': isDisabled }">
						<IntlFormatted
							:message-id="messages.countdownRemaining"
							:values="{
								days: timeLeftCountdown.days,
								hours: timeLeftCountdown.hours,
								minutes: timeLeftCountdown.minutes,
								seconds: timeLeftCountdown.seconds,
							}"
						>
							<template #days-count="{ children }">
								<span class="text-medal-orange"><component :is="() => children" /></span>
							</template>
							<template #hours-count="{ children }">
								<span class="text-medal-orange"><component :is="() => children" /></span>
							</template>
							<template #minutes-count="{ children }">
								<span class="text-medal-orange"><component :is="() => children" /></span>
							</template>
							<template #seconds-count="{ children }">
								<span class="text-medal-orange"><component :is="() => children" /></span>
							</template>
						</IntlFormatted>
					</span>
				</div>

				<div
					v-if="projectData?.title"
					class="m-0 flex flex-row items-center gap-2 text-sm font-medium"
					:class="{ 'opacity-50': isDisabled }"
				>
					<Avatar
						:src="iconUrl"
						no-shadow
						style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
						:alt="formatMessage(messages.serverIconAlt)"
					/>
					{{ formatMessage(messages.usingProjectLabel, { projectTitle: projectData?.title }) }}
				</div>

				<div
					v-if="isConfiguring"
					class="flex min-w-0 items-center gap-2 truncate text-sm font-medium text-blue h-[28px] w-max"
				>
					<SparklesIcon class="size-5 shrink-0 font-semibold" />
					{{ formatMessage(messages.newServerLabel) }}
				</div>
				<ServerInfoLabels
					v-else
					:server-data="{ game, mc_version, loader, loader_version, net }"
					:server-id="server_id"
					:show-game-label="showGameLabel"
					:show-loader-label="showLoaderLabel"
					:linked="false"
					:class="{ 'opacity-50': isDisabled }"
					class="flex w-full flex-row flex-wrap items-center gap-2 text-primary *:hidden sm:flex-row sm:*:flex"
				/>
			</div>

			<div class="z-10 ml-auto">
				<ButtonStyled color="medal-promo" type="outlined" size="large">
					<button class="my-auto" data-server-listing-button @click="handleUpgrade">
						<RocketIcon /> {{ formatMessage(messages.upgradeButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div
			v-if="status === 'suspended' && suspension_reason === 'upgrading'"
			class="server-listing-notice"
		>
			<div class="flex gap-2">
				{{ formatMessage(messages.upgradingNotice) }}
			</div>
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
			class="server-listing-notice"
		>
			<div>{{ formatMessage(messages.medalTrialEndedNotice) }}</div>
		</div>
		<div v-else-if="status === 'suspended' && suspension_reason" class="server-listing-notice">
			<div>
				{{
					formatMessage(messages.suspendedWithReasonNotice, {
						reason: suspension_reason,
					})
				}}
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div v-else-if="status === 'suspended'" class="server-listing-notice">
			<div>{{ formatMessage(messages.suspendedNotice) }}</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { LockIcon, RocketIcon, SparklesIcon, SpinnerIcon } from '@modrinth/assets'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import dayjsDuration from 'dayjs/plugin/duration'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectModrinthClient } from '../../../providers/api-client'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import CopyCode from '../../base/CopyCode.vue'
import IntlFormatted from '../../base/IntlFormatted.vue'
import ServerInfoLabels from '../labels/ServerInfoLabels.vue'
import MedalBackgroundImage from './MedalBackgroundImage.vue'

dayjs.extend(dayjsDuration)

type MedalServerListingProps = {
	server_id: string
	name: string
	status: Archon.Servers.v0.Status
	suspension_reason?: Archon.Servers.v0.SuspensionReason | null
	game?: Archon.Servers.v0.Game
	mc_version?: string | null
	loader?: Archon.Servers.v0.Loader | null
	loader_version?: string | null
	net?: Archon.Servers.v0.Net
	upstream?: Archon.Servers.v0.Upstream | null
	flows?: Archon.Servers.v0.Flows
	medal_expires?: string
}

const props = defineProps<MedalServerListingProps>()

const emit = defineEmits<{ (e: 'upgrade'): void }>()
const { formatMessage } = useVIntl()

const client = injectModrinthClient()
const router = useRouter()

// const isNuxt = computed(() => client instanceof NuxtModrinthClient)

const showGameLabel = computed(() => !!props.game)
const showLoaderLabel = computed(() => !!props.loader)
const isConfiguring = computed(() => props.flows?.intro)
const isUpgrading = computed(
	() => props.status === 'suspended' && props.suspension_reason === 'upgrading',
)
const isDisabled = computed(() => props.status === 'suspended')
const hasNotice = computed(() => props.status === 'suspended')

const { data: projectData } = useQuery({
	queryKey: ['server-project', props.server_id, props.upstream?.project_id],
	queryFn: async () => {
		if (!props.upstream?.project_id) return null
		return await client.labrinth.projects_v2.get(props.upstream.project_id)
	},
	enabled: !!props.upstream?.project_id,
})

const iconUrl = computed(() => projectData.value?.icon_url || undefined)

const messages = defineMessages({
	countdownRemaining: {
		id: 'servers.medal-listing.countdown.remaining',
		defaultMessage:
			'<days-count>{days}</days-count> {days, plural, one {day} other {days}} <hours-count>{hours}</hours-count> {hours, plural, one {hour} other {hours}} <minutes-count>{minutes}</minutes-count> {minutes, plural, one {minute} other {minutes}} <seconds-count>{seconds}</seconds-count> {seconds, plural, one {second} other {seconds}} remaining...',
	},
	serverIconAlt: {
		id: 'servers.medal-listing.server-icon-alt',
		defaultMessage: 'Server icon',
	},
	usingProjectLabel: {
		id: 'servers.medal-listing.using-project-label',
		defaultMessage: 'Using {projectTitle}',
	},
	newServerLabel: {
		id: 'servers.medal-listing.new-server-label',
		defaultMessage: 'New server',
	},
	upgradeButton: {
		id: 'servers.medal-listing.upgrade-button',
		defaultMessage: 'Upgrade',
	},
	upgradingNotice: {
		id: 'servers.medal-listing.notice.upgrading',
		defaultMessage:
			"Your server's hardware is currently being upgraded and will be back online shortly.",
	},
	medalTrialEndedNotice: {
		id: 'servers.medal-listing.notice.medal-trial-ended',
		defaultMessage:
			'Your Medal server trial has ended and your server has been suspended. Please upgrade to continue using your server.',
	},
	suspendedWithReasonNotice: {
		id: 'servers.medal-listing.notice.suspended-with-reason',
		defaultMessage:
			'Your server has been suspended: {reason}. Please update your billing information or contact Modrinth Support for more information.',
	},
	suspendedNotice: {
		id: 'servers.medal-listing.notice.suspended',
		defaultMessage:
			'Your server has been suspended. Please update your billing information or contact Modrinth Support for more information.',
	},
})

const timeLeftCountdown = ref({ days: 0, hours: 0, minutes: 0, seconds: 0 })
const expiryDate = computed(() => (props.medal_expires ? dayjs(props.medal_expires) : null))

function handleUpgrade(event: Event) {
	event.stopPropagation()
	emit('upgrade')
}

function navigateToServer(event: MouseEvent | KeyboardEvent) {
	if (isDisabled.value) return

	const target = event.target
	if (
		target instanceof HTMLElement &&
		target.closest('[data-subdomain-label], [data-server-listing-button]')
	) {
		return
	}

	router.push(`/hosting/manage/${props.server_id}`)
}

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

watch(expiryDate, () => updateCountdown(), { immediate: true })

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

.text-medal-orange {
	color: var(--medal-promotion-text-orange);
	font-weight: bold;
}

.border-medal-orange {
	border-color: var(--medal-promotion-bg-orange);
}

.server-listing-notice {
	@apply relative flex w-full rounded-b-2xl border-[1px] border-t-0 border-solid p-4 flex-col gap-4 border-surface-4 bg-bg-raised text-primary;
}

.hoverable:hover:not(:has([data-subdomain-label]:hover, [data-server-listing-button]:hover))
	.medal-promotion {
	filter: brightness(1.05) saturate(1.1);
}

.pressable:active:not(:has([data-subdomain-label]:active, [data-server-listing-button]:active)) {
	transform: scale(0.985);
}
</style>
