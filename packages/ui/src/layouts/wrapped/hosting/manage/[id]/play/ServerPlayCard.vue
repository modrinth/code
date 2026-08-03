<template>
	<section
		class="flex flex-col gap-4 rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-5"
	>
		<div class="flex flex-col gap-8 md:flex-row md:items-center">
			<div class="flex min-w-0 flex-1 flex-col gap-0.5">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.playWithAppTitle) }}
				</h2>
				<p class="m-0 text-base leading-6 text-primary">
					{{ formatMessage(messages.playWithAppDescription) }}
				</p>
			</div>
			<div class="flex shrink-0 flex-col gap-2 md:w-[164px]">
				<ButtonStyled color="brand">
					<button class="!h-10 w-full" @click="emit('play')">
						<PlayIcon aria-hidden="true" />
						{{ formatMessage(messages.playServerButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button class="!h-10 w-full" @click="emit('invite')">
						<UserPlusIcon aria-hidden="true" />
						{{ formatMessage(messages.invitePlayersButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div class="flex flex-col gap-3">
			<div class="h-px w-full bg-surface-4"></div>
			<div class="flex flex-col gap-0.5">
				<h2 class="m-0 text-xl font-semibold text-contrast">
					{{ formatMessage(messages.differentLauncherTitle) }}
				</h2>
				<p class="m-0 text-base leading-6 text-primary">
					{{ formatMessage(messages.differentLauncherDescription) }}
				</p>
			</div>
			<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
				<div class="joined-buttons shrink-0">
					<ButtonStyled>
						<a
							v-if="modpackDownloadUrl"
							class="!h-10"
							:href="modpackDownloadUrl"
							:download="modpackFilename"
						>
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(messages.downloadModpackButton) }}
						</a>
						<button v-else type="button" class="!h-10">
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(messages.downloadModpackButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<a
							v-if="modpackDownloadUrl"
							v-tooltip="formatMessage(messages.downloadModpackButton)"
							class="!h-10 !w-10 !px-0"
							:href="modpackDownloadUrl"
							:download="modpackFilename"
							:aria-label="formatMessage(messages.downloadModpackButton)"
						>
							<ChevronDownIcon aria-hidden="true" />
						</a>
						<button
							v-else
							v-tooltip="formatMessage(messages.downloadModpackButton)"
							type="button"
							class="!h-10 !w-10 !px-0"
							:aria-label="formatMessage(messages.downloadModpackButton)"
						>
							<ChevronDownIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
				<div
					class="flex h-10 min-w-0 items-center justify-between gap-3 rounded-xl bg-surface-2 px-4 sm:w-[313px]"
				>
					<span class="min-w-0 truncate font-semibold text-primary">{{ address }}</span>
					<button
						v-tooltip="formatMessage(copied ? messages.copiedAddress : messages.copyAddress)"
						type="button"
						class="flex shrink-0 cursor-pointer items-center border-0 bg-transparent p-0 text-primary"
						:aria-label="formatMessage(copied ? messages.copiedAddress : messages.copyAddress)"
						@click="copyAddress"
					>
						<CheckIcon v-if="copied" class="size-5 text-brand" aria-hidden="true" />
						<CopyIcon v-else class="size-5" aria-hidden="true" />
					</button>
				</div>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	CopyIcon,
	DownloadIcon,
	PlayIcon,
	UserPlusIcon,
} from '@modrinth/assets'
import { ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const props = defineProps<{
	address: string
	modpackDownloadUrl?: string
	modpackFilename?: string
}>()

const emit = defineEmits<{
	play: []
	invite: []
}>()

const { formatMessage } = useVIntl()
const copied = ref(false)

const messages = defineMessages({
	playWithAppTitle: {
		id: 'servers.play.card.app.title',
		defaultMessage: 'Play with Modrinth App',
	},
	playWithAppDescription: {
		id: 'servers.play.card.app.description',
		defaultMessage:
			'The easiest way to play is with the Modrinth App, which automatically installs all the required content for you and the players on your server.',
	},
	playServerButton: {
		id: 'servers.play.card.app.play-button',
		defaultMessage: 'Play server',
	},
	invitePlayersButton: {
		id: 'servers.play.card.app.invite-button',
		defaultMessage: 'Invite players',
	},
	differentLauncherTitle: {
		id: 'servers.play.card.launcher.title',
		defaultMessage: 'Using a Different Launcher?',
	},
	differentLauncherDescription: {
		id: 'servers.play.card.launcher.description',
		defaultMessage:
			'Install the required content using your preferred launcher, then add the server address in-game to join.',
	},
	downloadModpackButton: {
		id: 'servers.play.card.launcher.download-button',
		defaultMessage: 'Download modpack',
	},
	copyAddress: {
		id: 'servers.play.card.address.copy',
		defaultMessage: 'Copy server address',
	},
	copiedAddress: {
		id: 'servers.play.card.address.copied',
		defaultMessage: 'Server address copied',
	},
})

async function copyAddress() {
	await navigator.clipboard.writeText(props.address)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 2000)
}
</script>
