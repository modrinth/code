<template>
	<div v-if="open" class="open-in-app-modal">
		<div :class="{ shown: visible }" class="fullscreen-overlay" @click="hide" />
		<div class="modal-content" :class="{ shown: visible }">
			<div class="flex flex-col items-center gap-6">
				<div class="flex flex-col gap-6">
					<div v-if="countdown > 0" class="countdown-container">
						<svg class="countdown-svg" viewBox="0 0 100 100">
							<circle
								class="stroke-surface-4"
								cx="50"
								cy="50"
								r="45"
								fill="none"
								stroke-width="6"
							/>
							<circle
								class="countdown-progress"
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
						<span class="countdown-number">{{ countdown }}</span>
					</div>

					<h2 class="m-0 text-3xl font-bold text-contrast text-center">
						{{ formatMessage(messages.openingApp) }}
					</h2>

					<div
						class="flex flex-col items-center gap-4 bg-surface-3 rounded-3xl border border-solid border-surface-5 p-6"
					>
						<div class="flex items-center gap-3 rounded-xl bg-surface-2 p-3 w-full">
							<Avatar :src="serverProject.icon" :alt="serverProject.name" size="48px" />
							<div class="flex flex-col gap-1">
								<span class="font-semibold text-contrast">{{ serverProject.name }}</span>
								<div class="flex items-center gap-2 text-secondary">
									<ServerOnlinePlayers :online="serverProject.numPlayers ?? 0" />
									<ServerRegion v-if="serverProject.region" :region="serverProject.region" />
								</div>
							</div>
						</div>
						<div class="flex flex-col text-left gap-3">
							<span class="font-semibold text-contrast">{{
								formatMessage(messages.whyUseApp)
							}}</span>

							<div class="flex flex-col gap-2">
								<div class="flex text-base gap-2 items-center">
									<div
										class="w-5 h-5 border border-solid rounded-full flex items-center justify-center border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitLaunch) }}</span>
								</div>
								<div class="flex text-base gap-2 items-center">
									<div
										class="w-5 h-5 border border-solid rounded-full flex items-center justify-center border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitInstall) }}</span>
								</div>
								<div class="flex text-base gap-2 items-center">
									<div
										class="w-5 h-5 border border-solid rounded-full flex items-center justify-center border-brand bg-brand-highlight text-brand"
									>
										<CheckIcon />
									</div>
									<span>{{ formatMessage(messages.benefitUpdate) }}</span>
								</div>
							</div>
						</div>
					</div>
				</div>

				<span v-if="countdown > 0" class="text-secondary">{{
					formatMessage(messages.openingAutomatically)
				}}</span>
				<div v-else class="grid grid-cols-2 gap-2 w-full">
					<ButtonStyled class="flex-1">
						<button @click="hide">
							<XIcon />
							{{ formatMessage(messages.close) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="green" class="flex-1">
						<a href="https://modrinth.com/app" target="_blank" rel="noopener noreferrer">
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
import { computed, nextTick, onUnmounted, ref } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { Avatar, ButtonStyled } from '../base'
import ServerOnlinePlayers from '../project/server/ServerOnlinePlayers.vue'
import ServerRegion from '../project/server/ServerRegion.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	openingApp: {
		id: 'modal.open-in-app.title',
		defaultMessage: 'Opening Modrinth App',
	},
	whyUseApp: {
		id: 'modal.open-in-app.why-use',
		defaultMessage: 'Why use the Modrinth App',
	},
	benefitLaunch: {
		id: 'modal.open-in-app.benefit.launch',
		defaultMessage: 'Launch the game straight into the server',
	},
	benefitInstall: {
		id: 'modal.open-in-app.benefit.install',
		defaultMessage: 'Automatically install required content',
	},
	benefitUpdate: {
		id: 'modal.open-in-app.benefit.update',
		defaultMessage: 'Keep files updated when the server changes',
	},
	openingAutomatically: {
		id: 'modal.open-in-app.opening-automatically',
		defaultMessage: 'The Modrinth App will open automatically...',
	},
	close: {
		id: 'modal.open-in-app.close',
		defaultMessage: 'Close',
	},
	getApp: {
		id: 'modal.open-in-app.get-app',
		defaultMessage: 'Get Modrinth App',
	},
})

export interface ServerProject {
	name: string
	slug?: string
	numPlayers?: number
	icon?: string
	statusOnline?: boolean
	region?: string
}

const open = ref(false)
const visible = ref(false)
const countdown = ref(3)
const countdownProgress = ref(1)
let countdownInterval: ReturnType<typeof setInterval> | null = null
let progressInterval: ReturnType<typeof setInterval> | null = null

const circumference = 2 * Math.PI * 45
const strokeDashoffset = computed(() => {
	return circumference * (1 - countdownProgress.value)
})

const serverProject = ref<ServerProject>({
	name: '',
	slug: '',
	numPlayers: 0,
	icon: undefined,
	statusOnline: false,
	region: '',
})
const appLink = computed(() => {
	return `modrinth://modpack/${serverProject.value.slug}`
})

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
		if (countdown.value <= 0) {
			stopCountdown()
		}
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

interface ShowOpenInAppOptions {
	serverProject: ServerProject
}

async function show(options: ShowOpenInAppOptions) {
	serverProject.value = options.serverProject

	await nextTick()

	window.open(appLink.value, '_self')
	open.value = true
	document.body.style.overflow = 'hidden'
	window.addEventListener('keydown', handleKeyDown)
	setTimeout(() => {
		visible.value = true
		startCountdown()
	}, 50)
}

function hide() {
	visible.value = false
	document.body.style.overflow = ''
	window.removeEventListener('keydown', handleKeyDown)
	stopCountdown()
	setTimeout(() => {
		open.value = false
	}, 300)
}

function handleKeyDown(event: KeyboardEvent) {
	if (event.key === 'Escape') {
		hide()
	}
}

onUnmounted(() => {
	stopCountdown()
})

defineExpose({ show, hide, open })
</script>

<style lang="scss" scoped>
.open-in-app-modal {
	position: fixed;
	inset: 0;
	z-index: 100;
	display: flex;
	justify-content: center;
	align-items: center;
}

.fullscreen-overlay {
	position: fixed;
	inset: 0;
	background: linear-gradient(to bottom, rgba(66, 131, 92, 0.23) 0%, rgba(17, 35, 43, 0.4) 97%);
	backdrop-filter: blur(12px);
	opacity: 0;
	transition: opacity 0.3s ease-out;
	cursor: pointer;

	&.shown {
		opacity: 1;
	}
}

.modal-content {
	position: relative;
	z-index: 1;
	padding: 2.5rem;
	opacity: 0;
	transform: scale(0.95);
	transition: all 0.3s ease-out;

	&.shown {
		opacity: 1;
		transform: scale(1);
	}
}

.countdown-container {
	position: relative;
	width: 120px;
	height: 120px;
	margin: 0 auto;
	display: flex;
	align-items: center;
	justify-content: center;
}

.countdown-svg {
	position: absolute;
	width: 100%;
	height: 100%;
	transform: rotate(-90deg);
}

.countdown-bg {
	stroke: var(--surface-4);
}

.countdown-progress {
	stroke: var(--color-green);
	transition: stroke-dashoffset 0.05s linear;
}

.countdown-number {
	font-size: 3rem;
	font-weight: 700;
	color: var(--color-contrast);
	z-index: 1;
}
</style>
