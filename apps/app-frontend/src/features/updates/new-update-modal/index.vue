<script setup lang="ts">
import { CircleSlashIcon, PlayIcon, UserPlusIcon, XIcon } from '@modrinth/assets'
import {
	AutoLink,
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, ref, useTemplateRef } from 'vue'
import { useRouter } from 'vue-router'

import videoUrl from '@/assets/modrinth-hosting-server-play-demo.webm'
import videoPoster from '@/assets/modrinth-hosting-server-play-demo.webp'
import { config } from '@/config'

import { useNewUpdateNotification } from './use-notification'

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const video = useTemplateRef<HTMLVideoElement>('video')
const videoStarted = ref(false)
const videoPlaying = ref(false)
const videoLoading = ref(false)
const videoDuration = ref(16)
const videoCurrentTime = ref(0)
const remainingTime = computed(() => {
	const seconds = videoStarted.value
		? Math.max(1, Math.ceil(videoDuration.value - videoCurrentTime.value))
		: Math.round(videoDuration.value)
	return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
})
let playbackAttempt = 0
const router = useRouter()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { notifyForVersion, invitePath } = useNewUpdateNotification(show)

const messages = defineMessages({
	badge: {
		id: 'app.server-sharing-update.badge',
		defaultMessage: 'New this update',
	},
	title: {
		id: 'app.server-sharing-update.title',
		defaultMessage: 'Share servers with friends',
	},
	description: {
		id: 'app.server-sharing-update.description',
		defaultMessage:
			'You can now share your server directly with friends in the app. When they accept, we’ll create an instance for them that is managed by your server.',
	},
	blog: {
		id: 'app.server-sharing-update.blog',
		defaultMessage: 'Read more about this release in the <link>server sharing</link> blog.',
	},
	skip: {
		id: 'app.server-sharing-update.skip',
		defaultMessage: 'Skip',
	},
	invite: {
		id: 'app.server-sharing-update.invite',
		defaultMessage: 'Invite players',
	},
	watch: {
		id: 'app.server-sharing-update.watch',
		defaultMessage: 'Watch',
	},
	resume: {
		id: 'app.server-sharing-update.resume',
		defaultMessage: 'Resume',
	},
	pause: {
		id: 'app.server-sharing-update.pause',
		defaultMessage: 'Pause video',
	},
})

function show() {
	stopVideo()
	videoStarted.value = false
	videoPlaying.value = false
	videoCurrentTime.value = 0
	if (video.value) video.value.currentTime = 0
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function watchVideo() {
	const player = video.value
	if (!player || videoLoading.value) return
	syncVideoState()
	if (hasVideoEnded(player)) player.currentTime = 0
	const attempt = ++playbackAttempt
	videoLoading.value = true
	try {
		await player.play()
		if (attempt === playbackAttempt) syncVideoState()
	} catch (error) {
		if (attempt === playbackAttempt) handleError(error)
	} finally {
		if (attempt === playbackAttempt) videoLoading.value = false
	}
}

function stopVideo() {
	playbackAttempt++
	videoLoading.value = false
	video.value?.pause()
	syncVideoState()
}

function hasVideoEnded(player: HTMLVideoElement) {
	return (
		player.ended ||
		(Number.isFinite(player.duration) &&
			player.duration > 0 &&
			player.currentTime >= player.duration)
	)
}

function syncVideoState() {
	const player = video.value
	if (!player) return
	if (Number.isFinite(player.duration)) videoDuration.value = player.duration
	videoCurrentTime.value = player.currentTime
	if (hasVideoEnded(player)) {
		playbackAttempt++
		videoLoading.value = false
		videoPlaying.value = false
		videoStarted.value = false
		return
	}
	videoPlaying.value = !player.paused && player.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA
	if (videoPlaying.value) videoStarted.value = true
}

useEventListener(window, 'focus', syncVideoState)
useEventListener(document, 'visibilitychange', () => {
	if (document.visibilityState === 'visible') syncVideoState()
})

async function invitePlayers() {
	await router.push(invitePath.value)
	hide()
}

defineExpose({ show, hide, notifyForVersion })
</script>

<template>
	<NewModal
		ref="modal"
		hide-header
		no-padding
		width="546px"
		max-width="calc(100vw - 2rem)"
		:aria-label="formatMessage(messages.title)"
		:on-hide="stopVideo"
		class="!overflow-y-auto !rounded-[20px] !bg-surface-3"
	>
		<div class="w-[544px] max-w-full">
			<div class="relative aspect-video overflow-hidden rounded-t-[19px] bg-black">
				<video
					ref="video"
					:src="videoUrl"
					:aria-label="formatMessage(messages.title)"
					preload="auto"
					playsinline
					disablepictureinpicture
					disableremoteplayback
					class="pointer-events-none absolute inset-0 size-full object-contain"
					@loadedmetadata="syncVideoState"
					@durationchange="syncVideoState"
					@timeupdate="syncVideoState"
					@playing="syncVideoState"
					@pause="syncVideoState"
					@ended="syncVideoState"
				/>
				<Transition
					enter-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					leave-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					enter-from-class="opacity-0"
					leave-to-class="opacity-0"
				>
					<img
						v-if="!videoStarted"
						:src="videoPoster"
						alt=""
						class="pointer-events-none absolute inset-0 size-full object-contain"
					/>
				</Transition>
				<button
					v-if="videoPlaying"
					type="button"
					:aria-label="formatMessage(messages.pause)"
					class="absolute inset-0 size-full cursor-pointer border-0 bg-transparent focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-brand"
					@click="stopVideo"
				/>
				<Transition
					enter-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					leave-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					enter-from-class="opacity-0"
					leave-to-class="opacity-0"
				>
					<div
						v-if="!videoPlaying"
						class="absolute inset-0 flex items-center justify-center bg-black/30"
					>
						<Button
							size="xl"
							:loading="videoLoading"
							class="!bg-[rgba(52,54,60,0.7)] !px-5 !font-semibold !text-white !shadow-[inset_0_0_0_1px_rgba(66,68,74,0.7)] [&>svg]:!text-[#b0bac5]"
							@click="watchVideo"
						>
							<PlayIcon aria-hidden="true" />
							{{ formatMessage(videoStarted ? messages.resume : messages.watch) }}
							<span class="text-[#b0bac5]">{{ remainingTime }}</span>
						</Button>
					</div>
				</Transition>
				<IconButton
					type="quiet"
					size="sm"
					:label="formatMessage(commonMessages.closeButton)"
					class="!absolute right-5 top-5 z-10"
					@click="hide"
				>
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>

			<section
				class="flex flex-col items-start gap-6 border-0 border-t border-solid border-surface-5 p-6"
			>
				<div
					class="flex h-8 items-center rounded-full border border-solid border-brand bg-brand-highlight px-2.5 text-sm font-medium leading-5 text-brand"
				>
					{{ formatMessage(messages.badge) }}
				</div>

				<div class="flex w-full flex-col gap-3">
					<h2 class="m-0 text-2xl font-semibold leading-6 text-contrast">
						{{ formatMessage(messages.title) }}
					</h2>
					<p class="m-0 text-base leading-6 text-primary">
						{{ formatMessage(messages.description) }}
					</p>
					<p class="m-0 text-base leading-6 text-primary">
						<IntlFormatted :message-id="messages.blog">
							<template #link="{ children }">
								<AutoLink
									:to="`${config.siteUrl}/news/article/server-sharing`"
									class="font-medium text-link hover:underline"
								>
									<component :is="() => children" />
								</AutoLink>
							</template>
						</IntlFormatted>
					</p>
				</div>

				<div class="flex flex-wrap items-center gap-2.5">
					<Button size="lg" @click="hide">
						<CircleSlashIcon aria-hidden="true" />
						{{ formatMessage(messages.skip) }}
					</Button>
					<Button type="colored" color="brand" size="lg" @click="invitePlayers">
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.invite) }}
					</Button>
				</div>
			</section>
		</div>
	</NewModal>
</template>
