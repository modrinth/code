<template>
	<div class="mx-auto flex w-fit max-w-[500px] shrink-0 flex-col items-start gap-4">
		<div class="flex flex-col gap-2 w-full">
			<h2 class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.welcomeTitle) }}
			</h2>
			<p class="m-0 text-base text-secondary">
				{{ formatMessage(messages.welcomeDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-4">
			<span class="text-base font-medium text-secondary">
				{{ formatMessage(messages.setupStepsHeading) }}
			</span>

			<div class="rounded-[20px] border border-solid border-surface-5 bg-surface-3 p-5">
				<div class="flex flex-col">
					<div v-for="(step, i) in steps" :key="i" class="flex gap-3">
						<div class="flex w-10 shrink-0 flex-col items-center">
							<div
								class="flex size-10 items-center justify-center rounded-full border border-solid border-surface-5 bg-surface-4"
							>
								<component :is="step.icon" class="size-6" />
							</div>
							<div
								v-if="i < steps.length - 1"
								class="my-2 flex-1 w-0.5 rounded-full bg-surface-5"
							/>
						</div>
						<div class="flex flex-col gap-1 pt-2" :class="i < steps.length - 1 ? 'pb-4' : ''">
							<span class="text-base font-semibold text-contrast">
								{{ i + 1 }}. {{ step.title }}
							</span>
							<span class="text-base text-secondary">
								{{ step.description }}
							</span>
						</div>
					</div>
				</div>
			</div>
		</div>

		<div class="w-full">
			<Button
				v-tooltip="!canSetup ? permissionDeniedMessage : undefined"
				type="colored"
				color="brand"
				size="xl"
				class="ml-auto"
				:disabled="!canSetup || !worldId"
				@click="openModal()"
			>
				{{ formatMessage(messages.setupServerButton) }} <RightArrowIcon />
			</Button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { GlobeIcon, PackageIcon, RightArrowIcon, UsersIcon } from '@modrinth/assets'
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import { injectModrinthServerContext, injectNotificationManager } from '#ui/providers'
import {
	injectServerOnboardingFlow,
	type ServerOnboardingRequest,
} from '#ui/providers/server-onboarding'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { canSetup, permissionDeniedMessage } = useServerPermissions()
const { serverId, worldId } = injectModrinthServerContext()
const flow = injectServerOnboardingFlow()
const route = useRoute()
const router = useRouter()
const props = defineProps<{ siteUrl: string }>()

const messages = defineMessages({
	welcomeTitle: {
		id: 'servers.setup.onboarding.welcome.title',
		defaultMessage: 'Welcome to Modrinth Hosting',
	},
	welcomeDescription: {
		id: 'servers.setup.onboarding.welcome.description',
		defaultMessage: "Your server is ready. Here's what you need to do to start playing!",
	},
	setupStepsHeading: {
		id: 'servers.setup.onboarding.steps.heading',
		defaultMessage: 'Setup your server (~2mins)',
	},
	setupServerButton: {
		id: 'servers.setup.onboarding.setup-server.button',
		defaultMessage: 'Setup server',
	},
	chooseWhatToPlayTitle: {
		id: 'servers.setup.onboarding.step.choose.title',
		defaultMessage: 'Choose what to play',
	},
	chooseWhatToPlayDescription: {
		id: 'servers.setup.onboarding.step.choose.description',
		defaultMessage:
			'Pick your favorite modpack from Modrinth, or choose a loader and add the mods you want.',
	},
	configureWorldTitle: {
		id: 'servers.setup.onboarding.step.configure-world.title',
		defaultMessage: 'Configure your world',
	},
	configureWorldDescription: {
		id: 'servers.setup.onboarding.step.configure-world.description',
		defaultMessage:
			'Set up your world just like singleplayer. Choose your gamemode and world seed.',
	},
	inviteFriendsTitle: {
		id: 'servers.setup.onboarding.step.invite-friends.title',
		defaultMessage: 'Invite your friends',
	},
	inviteFriendsDescription: {
		id: 'servers.setup.onboarding.step.invite-friends.description',
		defaultMessage:
			'Invite friends to your server with a link and let them join in one click from the Modrinth App.',
	},
})

async function openModal(project?: ServerOnboardingRequest['project']) {
	if (!canSetup.value || !worldId.value) return
	try {
		await flow.open({ serverId, worldId: worldId.value, siteUrl: props.siteUrl, project })
	} catch (error) {
		handleError(error as Error)
	}
}

onMounted(async () => {
	const resumeStage = route.query.resumeModal
	if (!resumeStage) return
	const projectId = route.query.mp_pid as string | undefined
	const versionId = route.query.mp_vid as string | undefined
	const name = route.query.mp_name as string | undefined
	await router.replace({ query: {} })
	if (resumeStage === 'setup-type' || resumeStage === 'modpack') {
		await openModal(
			resumeStage === 'modpack' && projectId && versionId
				? { projectId, versionId, name: name ?? '' }
				: undefined,
		)
	}
})

const steps = computed(() => [
	{
		icon: PackageIcon,
		title: formatMessage(messages.chooseWhatToPlayTitle),
		description: formatMessage(messages.chooseWhatToPlayDescription),
	},
	{
		icon: GlobeIcon,
		title: formatMessage(messages.configureWorldTitle),
		description: formatMessage(messages.configureWorldDescription),
	},
	{
		icon: UsersIcon,
		title: formatMessage(messages.inviteFriendsTitle),
		description: formatMessage(messages.inviteFriendsDescription),
	},
])
</script>
