<template>
	<div class="grid grid-cols-2 gap-8 items-center justify-center py-10 max-w-[760px]">
		<!-- Left column -->
		<div class="flex flex-col gap-8 items-start pr-8 shrink-0">
			<!-- Heading -->
			<div class="flex flex-col gap-2 items-start w-[300px]">
				<p class="text-3xl leading-9 font-semibold text-contrast">
					{{ formatMessage(messages.modrinthHostingLabel) }}
				</p>
				<p class="text-base font-normal text-primary">
					{{ formatMessage(messages.noServersDescription) }}
				</p>
			</div>

			<!-- Feature list -->
			<div class="flex flex-col gap-4 items-start w-full">
				<div class="flex gap-3 items-start">
					<div
						class="bg-surface-4 border border-solid border-surface-5 rounded-full shrink-0 size-8 flex items-center justify-center"
					>
						<PackageOpenIcon class="size-5 text-secondary" aria-hidden="true" />
					</div>
					<div class="flex flex-col gap-0.5">
						<p class="text-base font-semibold text-contrast">
							{{ formatMessage(messages.oneClickModInstallsTitle) }}
						</p>
						<p class="text-base font-normal text-primary">
							{{ formatMessage(messages.oneClickModInstallsDescription) }}
						</p>
					</div>
				</div>

				<div class="flex gap-3 items-start">
					<div
						class="bg-surface-4 border border-solid border-surface-5 rounded-full shrink-0 size-8 flex items-center justify-center overflow-hidden"
					>
						<GlobeIcon class="size-5 text-secondary" aria-hidden="true" />
					</div>
					<div class="flex flex-col gap-0.5">
						<p class="text-base font-semibold text-contrast">
							{{ formatMessage(messages.simpleSetupTitle) }}
						</p>
						<p class="text-base font-normal text-primary">
							{{ formatMessage(messages.simpleSetupDescription) }}
						</p>
					</div>
				</div>

				<div class="flex gap-3 items-start">
					<div
						class="bg-surface-4 border border-solid border-surface-5 rounded-full shrink-0 size-8 flex items-center justify-center overflow-hidden"
					>
						<UsersIcon class="size-5 text-secondary" aria-hidden="true" />
					</div>
					<div class="flex flex-col gap-0.5">
						<p class="text-base font-semibold text-contrast">
							{{ formatMessage(messages.playWithFriendsTitle) }}
						</p>
						<p class="text-base font-normal text-primary">
							{{ formatMessage(messages.playWithFriendsDescription) }}
						</p>
					</div>
				</div>
			</div>

			<!-- CTA section -->
			<div class="flex flex-col gap-6 items-start">
				<div class="flex flex-col gap-3 items-start">
					<ButtonStyled color="brand">
						<button @click="onClickNewServer?.()">
							<PlusIcon aria-hidden="true" />
							{{ formatMessage(messages.newServerButton) }}
						</button>
					</ButtonStyled>

					<AutoLink
						to="https://modrinth.com/hosting"
						target="_blank"
						class="flex items-center gap-1 hover:brightness-125"
					>
						{{ formatMessage(messages.learnMoreLink) }}
						<RightArrowIcon class="size-5 shrink-0" aria-hidden="true" />
					</AutoLink>
				</div>

				<template v-if="!loggedIn">
					<div class="h-px w-full bg-surface-5" />

					<div class="flex gap-3 items-center flex-wrap">
						<p class="text-base font-normal text-primary">
							{{ formatMessage(messages.alreadyHaveServerLabel) }}
						</p>
						<ButtonStyled>
							<button @click="onClickSignIn?.()">
								<LogInIcon aria-hidden="true" />
								{{ formatMessage(messages.signInButton) }}
							</button>
						</ButtonStyled>
					</div>
				</template>
			</div>
		</div>

		<!-- Right column - mod icon grid -->
		<div
			class="relative flex h-[617px] shrink-0 items-center justify-center overflow-hidden rounded-[40px] pointer-events-none select-none [mask-image:linear-gradient(to_bottom,black_0%,black_35%,transparent_100%)] [-webkit-mask-image:linear-gradient(to_bottom,black_0%,black_35%,transparent_100%)]"
		>
			<div class="rotate-[15deg]">
				<div class="flex flex-col gap-4">
					<div
						v-for="row in GRID_ROWS"
						:key="row"
						class="flex gap-4 items-center shrink-0"
						:class="animated ? (row % 2 === 1 ? 'drift-left' : 'drift-right relative left-14') : ''"
					>
						<div class="hidden drift-right drift-left"></div>
						<div
							v-for="col in GRID_COLS"
							:key="col"
							class="border border-surface-5 rounded-[20px] shrink-0 size-[112px] bg-surface-4 overflow-hidden"
						>
							<img :src="getGridImage(row - 1, col - 1)" alt="" class="size-full object-cover" />
						</div>
						<div
							v-for="col in GRID_COLS"
							:key="col"
							class="border border-surface-5 rounded-[20px] shrink-0 size-[112px] bg-surface-4 overflow-hidden"
						>
							<img :src="getGridImage(row - 1, col - 1)" alt="" class="size-full object-cover" />
						</div>
						<div
							v-for="col in GRID_COLS"
							:key="col"
							class="border border-surface-5 rounded-[20px] shrink-0 size-[112px] bg-surface-4 overflow-hidden"
						>
							<img :src="getGridImage(row - 1, col - 1)" alt="" class="size-full object-cover" />
						</div>
					</div>
				</div>
			</div>
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
import { AutoLink } from '@modrinth/ui'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import imgAircraft from './grid-images/aircraft.png'
import imgAlexs from "./grid-images/alex's.png"
import imgArtifacts from './grid-images/artifacts.png'
import imgBiomes from './grid-images/biomes.png'
import imgCatac from './grid-images/catac.png'
import imgCobble from './grid-images/cobble.png'
import imgComforts from './grid-images/comforts.png'
import imgCreate from './grid-images/create.png'
import imgCreate1 from './grid-images/create1.png'
import imgCreate2 from './grid-images/create2.png'
import imgCreate3 from './grid-images/create3.png'
import imgCreeper from './grid-images/creeper.png'
import imgFriends from './grid-images/friends.png'
import imgGeo from './grid-images/geo.png'
import imgNaturalist from './grid-images/naturalist.png'
import imgSeasons from './grid-images/seasons.png'
import imgTravellers from './grid-images/travellers.png'
import imgTree from './grid-images/tree.png'
import imgYum1 from './grid-images/yum1.png'
import imgYum2 from './grid-images/yum2.png'
import imgYum3 from './grid-images/yum3.png'
import imgYung from './grid-images/yung.png'

withDefaults(
	defineProps<{
		animated?: boolean
		onClickNewServer?: () => void
		onClickSignIn?: () => void
		loggedIn?: boolean
	}>(),
	{ animated: false },
)

const GRID_ROWS = 6
const GRID_COLS = 5
const { formatMessage } = useVIntl()

const messages = defineMessages({
	modrinthHostingLabel: {
		id: 'servers.list-empty.modrinth-hosting-label',
		defaultMessage: 'Modrinth Hosting',
	},
	noServersTitle: {
		id: 'servers.list-empty.no-servers-title',
		defaultMessage: 'No servers yet',
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
		defaultMessage: 'Pick your favourite mods and we handle the rest.',
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
		defaultMessage: 'Invite friends and get them set up right in the Modrinth App.',
	},
	newServerButton: {
		id: 'servers.list-empty.new-server-button',
		defaultMessage: 'New server',
	},
	learnMoreLink: {
		id: 'servers.list-empty.learn-more-link',
		defaultMessage: 'Learn more about Modrinth Hosting',
	},
	alreadyHaveServerLabel: {
		id: 'servers.list-empty.already-have-server-label',
		defaultMessage: 'Already have a server?',
	},
	signInButton: {
		id: 'servers.list-empty.sign-in-button',
		defaultMessage: 'Sign in',
	},
})

const GRID_IMAGES = [
	imgYum1,
	imgYum2,
	imgYum3,
	imgYung,
	imgCreeper,
	imgFriends,
	imgNaturalist,
	imgBiomes,
	imgCatac,
	imgCobble,
	imgGeo,
	imgCreate,
	imgCreate1,
	imgCreate2,
	imgCreate3,
	imgAircraft,
	imgArtifacts,
	imgComforts,
	imgTravellers,
	imgAlexs,
	imgSeasons,
	imgTree,
]

function getGridImage(row: number, col: number): string {
	return GRID_IMAGES[(row * GRID_COLS + col) % GRID_IMAGES.length]
}
</script>

<style scoped>
p {
	margin: 0;
}

@keyframes drift-right {
	from {
		transform: translateX(-33%);
	}
	to {
		transform: translateX(33%);
	}
}

@keyframes drift-left {
	from {
		transform: translateX(33%);
	}
	to {
		transform: translateX(-33%);
	}
}

.drift-left {
	animation: drift-left linear infinite alternate;
	animation-duration: 400s;
}

.drift-right {
	animation: drift-right linear infinite alternate;
	animation-duration: 400s;
}
</style>
