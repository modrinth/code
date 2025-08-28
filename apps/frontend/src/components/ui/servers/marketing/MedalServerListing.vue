<template>
	<div class="rounded-2xl shadow-xl">
		<div
			class="medal-promotion flex flex-row items-center overflow-x-hidden rounded-t-2xl p-4 transition-transform duration-100"
			:class="status === 'suspended' ? 'rounded-b-none border-b-0 opacity-75' : 'rounded-b-2xl'"
			data-pyro-server-listing
			:data-pyro-server-listing-id="server_id"
		>
			<MedalBackgroundImage />
			<AutoLink
				:to="status === 'suspended' ? '' : `/servers/manage/${props.server_id}`"
				class="z-10 flex flex-grow flex-row items-center overflow-x-hidden"
				:class="status !== 'suspended' && 'active:scale-95'"
			>
				<Avatar
					v-if="status !== 'suspended'"
					src="https://cdn-raw.modrinth.com/medal_icon.webp"
					size="64px"
					class="z-10"
				/>
				<div
					v-else
					class="bg-bg-secondary z-10 flex size-16 shrink-0 items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
				>
					<LockIcon class="size-12 text-secondary" />
				</div>

				<div class="z-10 ml-4 flex min-w-0 flex-col gap-2.5">
					<div class="flex flex-row items-center gap-2">
						<h2 class="m-0 truncate text-xl font-bold text-contrast">{{ name }}</h2>
						<ChevronRightIcon />

						<span class="truncate">
							<span class="text-medal-orange">
								{{ timeLeftCountdown.days }}
							</span>
							days
							<span class="text-medal-orange">
								{{ timeLeftCountdown.hours }}
							</span>
							hours
							<span class="text-medal-orange">
								{{ timeLeftCountdown.minutes }}
							</span>
							minutes
							<span class="text-medal-orange">
								{{ timeLeftCountdown.seconds }}
							</span>
							seconds remaining...
						</span>
					</div>

					<div
						v-if="projectData?.title"
						class="m-0 flex flex-row items-center gap-2 text-sm font-medium text-secondary"
					>
						<Avatar
							:src="iconUrl"
							no-shadow
							style="min-height: 20px; min-width: 20px; height: 20px; width: 20px"
							alt="Server Icon"
						/>
						Using {{ projectData?.title || 'Unknown' }}
					</div>

					<div
						v-if="isConfiguring"
						class="text-medal-orange flex min-w-0 items-center gap-2 truncate text-sm font-semibold"
					>
						<SparklesIcon class="size-5 shrink-0" /> New server
					</div>
					<ServerInfoLabels
						v-else
						:server-data="{ game, mc_version, loader, loader_version, net }"
						:show-game-label="showGameLabel"
						:show-loader-label="showLoaderLabel"
						:linked="false"
						class="pointer-events-none flex w-full flex-row flex-wrap items-center gap-4 text-secondary *:hidden sm:flex-row sm:*:flex"
					/>
				</div>
			</AutoLink>

			<div class="z-10 ml-auto">
				<ButtonStyled color="medal-promo" type="outlined" size="large">
					<button class="my-auto" @click="handleUpgrade"><RocketIcon /> Upgrade</button>
				</ButtonStyled>
			</div>
		</div>

		<div
			v-if="status === 'suspended' && suspension_reason === 'upgrading'"
			class="relative flex w-full flex-row items-center gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-blue bg-bg-blue p-4 text-sm font-bold text-contrast"
		>
			<PanelSpinner />
			Your server's hardware is currently being upgraded and will be back online shortly.
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason === 'cancelled'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your Medal server trial has ended and your server has
				been suspended. Please upgrade to continue to use your server.
			</div>
		</div>
		<div
			v-else-if="status === 'suspended' && suspension_reason"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your server has been suspended: {{ suspension_reason }}.
				Please update your billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
		<div
			v-else-if="status === 'suspended'"
			class="relative flex w-full flex-col gap-2 rounded-b-2xl border-[1px] border-t-0 border-solid border-bg-red bg-bg-red p-4 text-sm font-bold text-contrast"
		>
			<div class="flex flex-row gap-2">
				<PanelErrorIcon class="!size-5" /> Your server has been suspended. Please update your
				billing information or contact Modrinth Support for more information.
			</div>
			<CopyCode :text="`${props.server_id}`" class="ml-auto" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, LockIcon, RocketIcon, SparklesIcon } from '@modrinth/assets'
import { AutoLink, Avatar, ButtonStyled, CopyCode } from '@modrinth/ui'
import type { Project, Server } from '@modrinth/utils'
import dayjs from 'dayjs'
import dayjsDuration from 'dayjs/plugin/duration'

import MedalBackgroundImage from '~/components/ui/servers/marketing/MedalBackgroundImage.vue'

import PanelErrorIcon from '../icons/PanelErrorIcon.vue'
import PanelSpinner from '../PanelSpinner.vue'
import ServerInfoLabels from '../ServerInfoLabels.vue'

dayjs.extend(dayjsDuration)

const props = defineProps<Partial<Server>>()
const emit = defineEmits<{ (e: 'upgrade'): void }>()

const showGameLabel = computed(() => !!props.game)
const showLoaderLabel = computed(() => !!props.loader)

let projectData: Ref<Project | null>
if (props.upstream) {
	const { data } = await useAsyncData<Project>(
		`server-project-${props.server_id}`,
		async (): Promise<Project> => {
			const result = await useBaseFetch(`project/${props.upstream?.project_id}`)
			return result as Project
		},
	)
	projectData = data
} else {
	projectData = ref(null)
}

const iconUrl = computed(() => projectData.value?.icon_url || undefined)
const isConfiguring = computed(() => props.flows?.intro)

const timeLeftCountdown = ref({ days: 0, hours: 0, minutes: 0, seconds: 0 })
const expiryDate = computed(() => (props.medal_expires ? dayjs(props.medal_expires) : null))

function handleUpgrade(event: Event) {
	event.stopPropagation()
	emit('upgrade')
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
</style>
