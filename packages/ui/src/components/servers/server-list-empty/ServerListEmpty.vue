<template>
	<div class="flex w-full max-w-[56rem] h-full flex-col items-stretch justify-between gap-8 py-6">
		<div class="flex w-full flex-wrap grow items-center justify-center gap-2">
			<div class="mx-auto flex w-full max-w-[20rem] flex-col items-start gap-8">
				<div class="flex flex-col gap-2">
					<div class="flex items-center gap-3">
						<h1 class="m-0 text-3xl font-semibold leading-9 text-contrast">
							{{ formatMessage(messages.modrinthHostingLabel) }}
						</h1>
					</div>
					<p class="m-0 text-base text-primary">
						{{ formatMessage(messages.noServersDescription) }}
					</p>
				</div>

				<div class="flex w-full flex-col gap-6">
					<div v-for="feature in features" :key="feature.key" class="flex items-start gap-4">
						<div
							class="feature-icon relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-[0.875rem] border border-solid bg-surface-1 text-brand"
						>
							<div class="feature-icon-gradient absolute left-[-1px] top-[-1px] size-[6.25rem]" />
							<div class="feature-icon-shade absolute left-[-1px] top-[-1px] size-[6.25rem]" />
							<img
								:src="iconTexture"
								alt=""
								class="absolute left-1/2 top-1/2 h-[6.25rem] w-[9.8125rem] max-w-none -translate-x-1/2 -translate-y-1/2 object-cover opacity-40 mix-blend-luminosity"
							/>
							<component
								:is="feature.icon"
								class="feature-icon-glyph relative size-5"
								aria-hidden="true"
							/>
						</div>
						<div class="flex min-w-0 flex-col gap-0.5">
							<p class="m-0 text-lg font-semibold leading-6 text-contrast">
								{{ feature.title }}
							</p>
							<p class="m-0 text-sm leading-5 text-primary">
								{{ feature.description }}
							</p>
						</div>
					</div>
				</div>

				<div class="flex flex-wrap items-center gap-4">
					<Button type="colored" color="brand" size="lg" @click="onClickNewServer?.()">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.newServerButton) }}
					</Button>
					<AutoLink
						to="https://modrinth.com/hosting"
						target="_blank"
						class="flex items-center gap-1 hover:brightness-125 font-semibold"
					>
						{{ formatMessage(messages.learnMoreLink) }}
						<RightArrowIcon class="size-5 shrink-0" aria-hidden="true" />
					</AutoLink>
				</div>
			</div>

			<ServerListEmptyPreview />
		</div>

		<div v-if="!loggedIn" class="flex flex-col items-center gap-4 text-center">
			<p class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.alreadyHaveServerLabel) }}
			</p>
			<Button @click="onClickSignIn?.()">
				<LogInIcon aria-hidden="true" />
				{{ formatMessage(messages.signInButton) }}
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	GlobeIcon,
	LogInIcon,
	PackageOpenIcon,
	PlusIcon,
	RightArrowIcon,
	UsersIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import iconTexture from '#ui/assets/welcome/icon-texture.png'
import AutoLink from '#ui/components/base/AutoLink.vue'
import { Button } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import ServerListEmptyPreview from './ServerListEmptyPreview.vue'

defineProps<{
	onClickNewServer?: () => void
	onClickSignIn?: () => void
	loggedIn?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modrinthHostingLabel: {
		id: 'servers.list-empty.modrinth-hosting-label',
		defaultMessage: 'Modrinth Hosting',
	},
	noServersDescription: {
		id: 'servers.list-empty.no-servers-description',
		defaultMessage: 'Install mods, invite friends, and play together all from the Modrinth App.',
	},
	oneClickModInstallsTitle: {
		id: 'servers.list-empty.one-click-mod-installs-title',
		defaultMessage: 'One-click mod installs',
	},
	oneClickModInstallsDescription: {
		id: 'servers.list-empty.one-click-mod-installs-description',
		defaultMessage: 'Pick your favorite mods and we handle the rest.',
	},
	simpleSetupTitle: {
		id: 'servers.list-empty.simple-setup-title',
		defaultMessage: 'Simple setup',
	},
	simpleSetupDescription: {
		id: 'servers.list-empty.simple-setup-description',
		defaultMessage: 'Set up your server just like a single player world.',
	},
	playWithFriendsTitle: {
		id: 'servers.list-empty.play-with-friends-title',
		defaultMessage: 'Play with friends',
	},
	playWithFriendsDescription: {
		id: 'servers.list-empty.play-with-friends-description',
		defaultMessage: 'Invite friends to play your server with one-click play from the Modrinth App.',
	},
	newServerButton: {
		id: 'servers.list-empty.new-server-button',
		defaultMessage: 'New server',
	},
	learnMoreLink: {
		id: 'servers.list-empty.learn-more-link',
		defaultMessage: 'Learn more',
	},
	alreadyHaveServerLabel: {
		id: 'servers.list-empty.already-have-server-label',
		defaultMessage: 'Already have a server?',
	},
	signInButton: {
		id: 'servers.list-empty.sign-in-button',
		defaultMessage: 'Sign in to Modrinth',
	},
})

const features = computed(() => [
	{
		key: 'one-click-mod-installs',
		icon: PackageOpenIcon,
		title: formatMessage(messages.oneClickModInstallsTitle),
		description: formatMessage(messages.oneClickModInstallsDescription),
	},
	{
		key: 'simple-setup',
		icon: GlobeIcon,
		title: formatMessage(messages.simpleSetupTitle),
		description: formatMessage(messages.simpleSetupDescription),
	},
	{
		key: 'play-with-friends',
		icon: UsersIcon,
		title: formatMessage(messages.playWithFriendsTitle),
		description: formatMessage(messages.playWithFriendsDescription),
	},
])
</script>

<style scoped>
.feature-icon {
	border-color: color-mix(in srgb, var(--color-text-primary) 10%, transparent);
	box-shadow:
		0 0 0 1px color-mix(in srgb, var(--color-brand) 30%, var(--surface-1)),
		var(--shadow-card),
		0 0 3.75rem color-mix(in srgb, var(--color-brand) 10%, transparent);
}

.feature-icon-gradient {
	background: linear-gradient(180deg, var(--color-green-800) 0%, var(--color-green-950) 100%);
	opacity: 0.5;
}

.feature-icon-shade {
	background: linear-gradient(
		-14deg,
		color-mix(in srgb, var(--color-green-950) 37%, transparent) 8%,
		transparent 86%
	);
}

.feature-icon-glyph {
	filter: drop-shadow(0 1px 1px color-mix(in srgb, var(--color-green-950) 12%, transparent));
}
</style>
