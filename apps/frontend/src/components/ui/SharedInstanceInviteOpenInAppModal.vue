<template>
	<div v-if="open" class="fixed inset-0 z-[100] flex items-center justify-center">
		<div
			:class="visible ? 'opacity-100' : 'opacity-0'"
			class="fullscreen-overlay fixed inset-0 cursor-pointer backdrop-blur-xl transition-opacity duration-300 ease-out"
			@click="hide"
		/>
		<div
			class="relative z-[1] max-h-screen w-full max-w-[28rem] overflow-y-auto px-4 py-10 transition-all duration-300 ease-out"
			:class="visible ? 'scale-100 opacity-100' : 'scale-95 opacity-0'"
		>
			<div class="flex w-full flex-col items-center gap-6">
				<div class="flex w-full flex-col gap-6">
					<div
						v-if="countdown > 0"
						class="relative mx-auto flex size-[120px] items-center justify-center"
					>
						<svg class="absolute size-full -rotate-90" viewBox="0 0 100 100" aria-hidden="true">
							<circle
								class="stroke-surface-4"
								cx="50"
								cy="50"
								r="45"
								fill="none"
								stroke-width="6"
							/>
							<circle
								class="stroke-brand transition-[stroke-dashoffset] duration-[50ms] ease-linear"
								cx="50"
								cy="50"
								r="45"
								fill="none"
								stroke-width="6"
								:stroke-dasharray="circumference"
								:stroke-dashoffset="strokeDashoffset"
								stroke-linecap="round"
							/>
						</svg>
						<span class="z-[1] text-5xl font-bold text-contrast">{{ countdown }}</span>
					</div>

					<h2 class="m-0 text-center text-3xl font-bold text-contrast">
						{{ formatMessage(messages.openingApp) }}
					</h2>

					<div
						class="flex flex-col items-center gap-4 rounded-3xl border border-solid border-surface-5 bg-surface-3 p-6"
					>
						<div class="flex w-full items-center gap-3 rounded-xl bg-surface-2 p-3">
							<Avatar :src="null" :alt="instance.name" size="48px" />
							<div class="flex min-w-0 flex-col gap-1">
								<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
								<span class="flex items-center gap-2 font-medium text-primary">
									<Avatar
										:src="instance.inviterAvatar"
										:alt="instance.inviterName"
										size="24px"
										circle
										no-shadow
									/>
									<span>{{ instance.inviterName }}</span>
								</span>
								<span class="text-sm text-secondary">
									{{
										formatMessage(messages.instanceDetails, {
											gameVersion: instance.gameVersion,
											loaderVersion: instance.loaderVersion,
										})
									}}
								</span>
							</div>
						</div>
						<div class="flex flex-col gap-3 text-left">
							<span class="font-semibold text-contrast">{{
								formatMessage(messages.whyUseApp)
							}}</span>

							<div class="flex flex-col gap-2">
								<div class="flex items-center gap-2 text-base">
									<div
										class="flex size-5 shrink-0 items-center justify-center rounded-full border border-solid border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitJoin) }}</span>
								</div>
								<div class="flex items-center gap-2 text-base">
									<div
										class="flex size-5 shrink-0 items-center justify-center rounded-full border border-solid border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitInstall) }}</span>
								</div>
								<div class="flex items-center gap-2 text-base">
									<div
										class="flex size-5 shrink-0 items-center justify-center rounded-full border border-solid border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitUpdate) }}</span>
								</div>
							</div>
						</div>
					</div>
				</div>

				<span v-if="countdown > 0" class="text-center text-secondary">{{
					formatMessage(messages.openingAutomatically)
				}}</span>
				<div v-else class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">
					<ButtonStyled>
						<button class="flex-1" type="button" @click="hide">
							<XIcon />
							{{ formatMessage(commonMessages.closeButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<a
							class="flex-1"
							href="https://modrinth.com/app"
							target="_blank"
							rel="noopener noreferrer"
						>
							<DownloadIcon />
							{{ formatMessage(messages.getApp) }}
						</a>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, DownloadIcon, XIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, onUnmounted, ref } from 'vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	openingApp: {
		id: 'modal.shared-instance.open-in-app.title',
		defaultMessage: 'Opening Modrinth App',
	},
	whyUseApp: {
		id: 'modal.shared-instance.open-in-app.why-use',
		defaultMessage: 'Why use the Modrinth App',
	},
	instanceDetails: {
		id: 'modal.shared-instance.open-in-app.instance-details',
		defaultMessage: 'Minecraft {gameVersion} · Loader {loaderVersion}',
	},
	benefitJoin: {
		id: 'modal.shared-instance.open-in-app.benefit.join',
		defaultMessage: 'Join the shared instance in one click',
	},
	benefitInstall: {
		id: 'modal.shared-instance.open-in-app.benefit.install',
		defaultMessage: 'Automatically install the required game and loader',
	},
	benefitUpdate: {
		id: 'modal.shared-instance.open-in-app.benefit.update',
		defaultMessage: 'Keep shared content updated when the instance changes',
	},
	openingAutomatically: {
		id: 'modal.shared-instance.open-in-app.opening-automatically',
		defaultMessage: 'The Modrinth App will open automatically...',
	},
	getApp: {
		id: 'modal.shared-instance.open-in-app.get-app',
		defaultMessage: 'Get Modrinth App',
	},
})

export interface SharedInstanceInviteModalData {
	inviteId: string
	name: string
	inviterName: string
	inviterAvatar?: string | null
	gameVersion: string
	loaderVersion: string
}

const open = ref(false)
const visible = ref(false)
const countdown = ref(3)
const countdownProgress = ref(1)
const instance = ref<SharedInstanceInviteModalData>({
	inviteId: '',
	name: '',
	inviterName: '',
	inviterAvatar: null,
	gameVersion: '',
	loaderVersion: '',
})

let countdownInterval: ReturnType<typeof setInterval> | null = null
let progressInterval: ReturnType<typeof setInterval> | null = null
let showTimeout: ReturnType<typeof setTimeout> | null = null
let hideTimeout: ReturnType<typeof setTimeout> | null = null

const circumference = 2 * Math.PI * 45
const strokeDashoffset = computed(() => circumference * (1 - countdownProgress.value))
const appLink = computed(() => `modrinth://share/${encodeURIComponent(instance.value.inviteId)}`)

function startCountdown() {
	countdown.value = 3
	countdownProgress.value = 1

	const totalDuration = 3000
	const progressUpdateInterval = 16
	const progressDecrement = progressUpdateInterval / totalDuration

	progressInterval = setInterval(() => {
		countdownProgress.value = Math.max(0, countdownProgress.value - progressDecrement)
	}, progressUpdateInterval)

	countdownInterval = setInterval(() => {
		countdown.value--
		if (countdown.value <= 0) stopCountdown()
	}, 1000)
}

function stopCountdown() {
	if (countdownInterval) {
		clearInterval(countdownInterval)
		countdownInterval = null
	}
	if (progressInterval) {
		clearInterval(progressInterval)
		progressInterval = null
	}
}

async function show(options: { instance: SharedInstanceInviteModalData }) {
	instance.value = options.instance
	open.value = true
	document.body.style.overflow = 'hidden'
	window.addEventListener('keydown', handleKeyDown)

	await nextTick()
	window.open(appLink.value, '_self')

	showTimeout = setTimeout(() => {
		visible.value = true
		startCountdown()
	}, 50)
}

function hide() {
	visible.value = false
	document.body.style.overflow = ''
	window.removeEventListener('keydown', handleKeyDown)
	stopCountdown()
	if (showTimeout) clearTimeout(showTimeout)

	hideTimeout = setTimeout(() => {
		open.value = false
	}, 300)
}

function handleKeyDown(event: KeyboardEvent) {
	if (event.key === 'Escape') hide()
}

onUnmounted(() => {
	document.body.style.overflow = ''
	window.removeEventListener('keydown', handleKeyDown)
	stopCountdown()
	if (showTimeout) clearTimeout(showTimeout)
	if (hideTimeout) clearTimeout(hideTimeout)
})

defineExpose({ show, hide, open })
</script>

<style scoped>
.fullscreen-overlay {
	background: linear-gradient(to bottom, rgb(66 131 92 / 23%) 0%, rgb(17 35 43 / 40%) 97%);
}
</style>
