<script setup lang="ts">
import {
	CircleSlashIcon,
	PlayIcon,
	RightArrowIcon,
	SpinnerIcon,
	Volume2Icon,
	VolumeXIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref, shallowRef, useTemplateRef, watch } from 'vue'
import { useRouter } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import { Button, IconButton } from '#ui/components/base/buttons'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { serverPlayMigrationMessages } from '#ui/components/servers/play-migration/messages'
import ServerPlayMigrationFlow from '#ui/components/servers/play-migration/ServerPlayMigrationFlow.vue'
import type { ServerPlayMigrationSource } from '#ui/components/servers/play-migration/types'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import InvitePlayersContent from '../invite-players-modal/invite-players-content.vue'
import { injectServerInviteHandoff } from './handoff'
import { useVideoPlayer } from './use-video-player'

/**
 * Where "Continue" sends the user. On a server's play page the invite flow is handed off
 * into this modal; anywhere else the user first picks a server.
 */
export type ServerSharingUpdateTarget = {
	path: string
	playPage: boolean
}

const props = defineProps<{
	siteUrl: string
	videoSrc: string
	posterSrc: string
	/** Overrides the default description, which is written for the Modrinth App. */
	description?: string
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const video = useTemplateRef<HTMLVideoElement>('video')
const inviting = ref(false)
const stage = ref<'intro' | 'migration' | 'invite'>('intro')
const inviteHandoff = injectServerInviteHandoff()
const inviteBinding = computed(() => inviteHandoff.binding.value?.value ?? null)
const lastBinding = shallowRef<typeof inviteBinding.value>(null)
const displayBinding = computed(() => inviteBinding.value ?? lastBinding.value)
const migrationBinding = computed(() => inviteHandoff.migration.value)
const lastMigrationBinding = shallowRef<typeof migrationBinding.value>(null)
const displayMigrationBinding = computed(() => migrationBinding.value ?? lastMigrationBinding.value)
const router = useRouter()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const {
	started: videoStarted,
	playing: videoPlaying,
	loading: videoLoading,
	muted: videoMuted,
	remainingTime,
	play: watchVideo,
	stop: stopVideo,
	toggleMute,
	reset: resetVideo,
	sync: syncVideoState,
} = useVideoPlayer(video, handleError)
const target = ref<ServerSharingUpdateTarget>({ path: '/hosting/manage', playPage: false })

const messages = defineMessages({
	badge: {
		id: 'servers.sharing-update.badge',
		defaultMessage: 'New this update',
	},
	title: {
		id: 'servers.sharing-update.title',
		defaultMessage: 'Share servers with friends',
	},
	description: {
		id: 'servers.sharing-update.description',
		defaultMessage:
			'You can now share your server directly with friends in the app. When they accept, we’ll create an instance for them that is managed by your server.',
	},
	blog: {
		id: 'servers.sharing-update.blog',
		defaultMessage: 'Read more about this release in the <link>server sharing</link> blog.',
	},
	skip: {
		id: 'servers.sharing-update.skip',
		defaultMessage: 'Skip',
	},
	continue: {
		id: 'servers.sharing-update.continue',
		defaultMessage: 'Continue',
	},
	watch: {
		id: 'servers.sharing-update.watch',
		defaultMessage: 'Watch',
	},
	resume: {
		id: 'servers.sharing-update.resume',
		defaultMessage: 'Resume',
	},
	pause: {
		id: 'servers.sharing-update.pause',
		defaultMessage: 'Pause video',
	},
	mute: {
		id: 'servers.sharing-update.mute',
		defaultMessage: 'Mute video',
	},
	unmute: {
		id: 'servers.sharing-update.unmute',
		defaultMessage: 'Unmute video',
	},
})

function show(next: ServerSharingUpdateTarget) {
	target.value = next
	stage.value = 'intro'
	inviting.value = false
	resetVideo()
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function onHide() {
	stopVideo()
	inviteHandoff.cancel()
}

watch(inviteBinding, async (binding) => {
	if (!binding || !inviting.value) return
	lastBinding.value = binding
	await modal.value?.transitionContent(() => {
		stage.value = 'invite'
		inviting.value = false
	})
})

watch(migrationBinding, async (binding) => {
	if (!binding || !inviting.value) return
	lastMigrationBinding.value = binding
	await modal.value?.transitionContent(() => {
		stage.value = 'migration'
		inviting.value = false
	})
})

async function backToIntro() {
	inviteHandoff.cancel()
	await modal.value?.transitionContent(() => {
		stage.value = 'intro'
	})
}

function startMigration(source: ServerPlayMigrationSource) {
	void displayMigrationBinding.value?.start(source)
	hide()
}

watch(
	() => inviteHandoff.requested.value,
	(requested) => {
		if (!requested && stage.value === 'intro') inviting.value = false
	},
)

async function invitePlayers() {
	if (inviting.value) return
	const { path, playPage } = target.value
	if (!playPage) {
		inviteHandoff.awaitingServer.value = true
		await router.push(path)
		hide()
		return
	}
	inviting.value = true
	inviteHandoff.request()
	try {
		await router.push(path)
	} catch (error) {
		inviteHandoff.cancel()
		inviting.value = false
		handleError(error as Error)
	}
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		hide-header
		no-padding
		width="546px"
		max-width="calc(100vw - 2rem)"
		:aria-label="formatMessage(messages.title)"
		:on-hide="onHide"
		class="!overflow-y-auto !rounded-[20px] !bg-surface-3"
	>
		<div v-if="stage === 'intro'" class="w-[544px] max-w-full">
			<div class="relative aspect-video overflow-hidden rounded-t-[19px] bg-black">
				<video
					ref="video"
					:src="videoSrc"
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
						:src="posterSrc"
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
				<Transition
					enter-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					leave-active-class="transition-opacity duration-300 ease-out motion-reduce:transition-none"
					enter-from-class="opacity-0"
					leave-to-class="opacity-0"
				>
					<Button
						v-if="videoPlaying"
						size="sm"
						:aria-label="formatMessage(videoMuted ? messages.unmute : messages.mute)"
						class="!absolute bottom-5 right-5 z-10 !size-8 !rounded-full !bg-[rgba(52,54,60,0.7)] !p-0 !text-white !shadow-[inset_0_0_0_1px_rgba(66,68,74,0.7)] [&>svg]:!text-[#b0bac5]"
						@click="toggleMute"
					>
						<VolumeXIcon v-if="videoMuted" aria-hidden="true" />
						<Volume2Icon v-else aria-hidden="true" />
					</Button>
				</Transition>
				<IconButton
					type="quiet"
					size="sm"
					:label="formatMessage(commonMessages.closeButton)"
					class="!absolute right-5 top-5 z-10 !size-8 !rounded-full !p-0 hover:!bg-[rgba(52,54,60,0.7)] hover:!text-white hover:!shadow-[inset_0_0_0_1px_rgba(66,68,74,0.7)] [&>svg]:hover:!text-[#b0bac5]"
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
						{{ description ?? formatMessage(messages.description) }}
					</p>
					<p class="m-0 text-base leading-6 text-primary">
						<IntlFormatted :message-id="messages.blog">
							<template #link="{ children }">
								<AutoLink
									:to="`${props.siteUrl}/news/article/server-sharing`"
									class="font-medium text-link hover:underline"
								>
									<component :is="() => children" />
								</AutoLink>
							</template>
						</IntlFormatted>
					</p>
				</div>

				<div class="flex flex-wrap items-center gap-2.5">
					<Button size="lg" :disabled="inviting" @click="hide">
						<CircleSlashIcon aria-hidden="true" />
						{{ formatMessage(messages.skip) }}
					</Button>
					<Button type="colored" color="brand" size="lg" :loading="inviting" @click="invitePlayers">
						<SpinnerIcon v-if="inviting" class="animate-spin" aria-hidden="true" />
						<RightArrowIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.continue) }}
					</Button>
				</div>
			</section>
		</div>
		<div v-else-if="stage === 'migration' && displayMigrationBinding" class="w-[544px] max-w-full">
			<div
				class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 p-6"
			>
				<h2 class="m-0 min-w-0 text-2xl font-semibold text-contrast">
					{{ formatMessage(serverPlayMigrationMessages.title) }}
				</h2>
				<IconButton :label="formatMessage(commonMessages.closeButton)" @click="hide">
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>
			<ServerPlayMigrationFlow
				class="!w-full p-6"
				:kind="displayMigrationBinding.kind"
				:modpack-name="displayMigrationBinding.modpackName"
				:has-uploaded-mrpack="displayMigrationBinding.hasUploadedMrpack"
				:world-id="displayMigrationBinding.worldId"
				@back="backToIntro"
				@confirm="startMigration"
			/>
		</div>
		<div v-else-if="displayBinding" class="w-[544px] max-w-full">
			<div
				class="flex items-center justify-between gap-4 border-0 border-b border-solid border-surface-5 p-6"
			>
				<h2 class="m-0 min-w-0 text-2xl font-semibold text-contrast">
					{{ displayBinding.header }}
				</h2>
				<IconButton :label="formatMessage(commonMessages.closeButton)" @click="hide">
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>
			<InvitePlayersContent
				v-bind="displayBinding.props"
				@invite="displayBinding.onInvite"
				@cancel="displayBinding.onCancel"
			/>
		</div>
	</NewModal>
</template>
