<template>
	<div class="mx-auto flex w-fit flex-col items-start gap-4 mt-16 max-w-[500px]">
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
						<div :class="['flex flex-col gap-1 pt-2', i < steps.length - 1 ? 'pb-[44px]' : '']">
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
			<ButtonStyled v-if="uploading" size="large">
				<button class="ml-auto" disabled>
					<SpinnerIcon class="animate-spin" />
					{{ formatMessage(messages.uploadingProgress, { percent: uploadPercent }) }}
				</button>
			</ButtonStyled>
			<ButtonStyled v-else color="brand" size="large">
				<button
					v-tooltip="!canSetup ? permissionDeniedMessage : undefined"
					class="ml-auto"
					:disabled="!canSetup"
					@click="openModal"
				>
					{{ formatMessage(messages.setupServerButton) }} <RightArrowIcon />
				</button>
			</ButtonStyled>
		</div>

		<CreationFlowModal
			ref="modalRef"
			type="world"
			:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
			:show-snapshot-toggle="true"
			:search-modpacks="searchModpacks"
			:get-project-versions="getProjectVersions"
			:finish-disabled="!canSetup"
			:finish-disabled-tooltip="!canSetup ? permissionDeniedMessage : undefined"
			@hide="() => {}"
			@browse-modpacks="onBrowseModpacks"
			@create="onCreate"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { GlobeIcon, PackageIcon, RightArrowIcon, SpinnerIcon, UsersIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useServerPermissions,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { CreationFlowContextValue } from '#ui/components'
import { CreationFlowModal } from '#ui/components'
import { injectModrinthServerContext } from '#ui/providers'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { canSetup, permissionDeniedMessage } = useServerPermissions()

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
		defaultMessage: 'Create instance (~2 mins)',
	},
	uploadingProgress: {
		id: 'servers.setup.onboarding.uploading.progress',
		defaultMessage: 'Uploading ({percent, number}%)',
	},
	setupServerButton: {
		id: 'servers.setup.onboarding.setup-server.button',
		defaultMessage: 'Create instance',
	},
	modpackUploadFailedTitle: {
		id: 'servers.setup.onboarding.modpack-upload-failed.title',
		defaultMessage: 'Modpack upload failed',
	},
	modpackUploadFailedText: {
		id: 'servers.setup.onboarding.modpack-upload-failed.text',
		defaultMessage: 'An unexpected error occurred while uploading. Please try again later.',
	},
	installationFailedTitle: {
		id: 'servers.setup.onboarding.installation-failed.title',
		defaultMessage: 'Installation failed',
	},
	installationFailedText: {
		id: 'servers.setup.onboarding.installation-failed.text',
		defaultMessage: 'An unexpected error occurred while installing. Please try again later.',
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
	configureInstanceTitle: {
		id: 'servers.setup.onboarding.step.configure-instance.title',
		defaultMessage: 'Configure your instance',
	},
	configureInstanceDescription: {
		id: 'servers.setup.onboarding.step.configure-instance.description',
		defaultMessage:
			'Set up your instance just like singleplayer. Choose your gamemode and world seed.',
	},
	inviteFriendsTitle: {
		id: 'servers.setup.onboarding.step.invite-friends.title',
		defaultMessage: 'Invite your friends',
	},
	inviteFriendsDescription: {
		id: 'servers.setup.onboarding.step.invite-friends.description',
		defaultMessage:
			"Share your server with friends by copying the address and letting them know which mods they'll need to join.",
	},
})

async function searchModpacks(query: string, limit: number = 10) {
	return client.labrinth.projects_v2.search({
		query: query || undefined,
		facets: [['project_type:modpack'], ['client_side:required'], ['server_side:required']],
		limit,
	})
}

async function getProjectVersions(projectId: string) {
	const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
	return versions.map((v) => ({ id: v.id }))
}
const { serverId, server } = injectModrinthServerContext()
const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()

const props = withDefaults(
	defineProps<{
		browseModpacks?: (args: {
			serverId: string
			worldId: string | null
			from: 'onboarding'
		}) => void | Promise<void>
	}>(),
	{
		browseModpacks: undefined,
	},
)

const modalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)

const uploading = ref(false)
const uploadedBytes = ref(0)
const totalBytes = ref(0)
const uploadPercent = computed(() =>
	totalBytes.value > 0 ? Math.round((uploadedBytes.value / totalBytes.value) * 100) : 0,
)

const openModal = () => {
	if (!canSetup.value) return
	modalRef.value?.show()
}

onBeforeUnmount(() => modalRef.value?.hide())

function onBrowseModpacks() {
	if (!canSetup.value) return

	if (props.browseModpacks) {
		props.browseModpacks({
			serverId,
			worldId: null,
			from: 'onboarding',
		})
		return
	}

	router.push({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'onboarding' },
	})
}

onMounted(async () => {
	if (!canSetup.value && route.query.resumeModal) {
		router.replace({ query: {} })
		return
	}

	if (route.query.resumeModal === 'setup-type') {
		router.replace({ query: {} })
		openModal()
		return
	}

	if (route.query.resumeModal === 'modpack') {
		const mpPid = route.query.mp_pid as string | undefined
		const mpVid = route.query.mp_vid as string | undefined
		const mpName = route.query.mp_name as string | undefined

		router.replace({ query: {} })
		openModal()
		await nextTick()

		const ctx = modalRef.value?.ctx
		if (ctx && mpPid && mpVid) {
			ctx.setupType.value = 'modpack'
			ctx.modpackSelection.value = {
				projectId: mpPid,
				versionId: mpVid,
				name: mpName ?? '',
			}
			ctx.modal.value?.setStage('final-config')
		} else {
			ctx?.setSetupType('modpack')
		}
	}
})

async function finalizeSetup(createdWorldId: string) {
	modalRef.value?.hide()
	server.value.flows = { intro: false }
	await client.archon.servers_v1.endIntro(serverId)
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['servers', 'worlds', 'summary', 'v1', serverId] }),
	])
	await router.push(
		`/hosting/manage/${encodeURIComponent(serverId)}/instances/${encodeURIComponent(createdWorldId)}`,
	)
}

function toApiLoader(loader: string | null | undefined): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return (loader ?? 'vanilla') as Archon.Content.v1.Modloader
}

const onCreate = async (config: CreationFlowContextValue) => {
	if (!canSetup.value) {
		config.loading.value = false
		return
	}

	const properties = config.buildProperties()

	try {
		if (config.setupType.value === 'modpack' && config.modpackFile.value) {
			const createdWorld = await client.archon.servers_v1.createWorld(serverId, {
				name: config.worldName.value.trim(),
				properties,
				content: createBareWorldContent(config),
			})
			modalRef.value?.hide()
			uploading.value = true
			uploadedBytes.value = 0
			totalBytes.value = config.modpackFile.value.size
			const handle = client.kyros.content_v1.uploadModpackFile(
				createdWorld.id,
				config.modpackFile.value,
				properties,
				{
					softOverride: false,
					onProgress: ({ loaded, total }) => {
						uploadedBytes.value = loaded
						totalBytes.value = total
					},
				},
			)
			await handle.promise
			server.value.status = 'installing'
			await finalizeSetup(createdWorld.id)
			return
		}

		const createdWorld = await client.archon.servers_v1.createWorld(serverId, {
			name: config.worldName.value.trim(),
			properties,
			content: createWorldContent(config),
		})
		server.value.status = 'installing'
		await finalizeSetup(createdWorld.id)
	} catch {
		addNotification({
			title:
				config.setupType.value === 'modpack' && config.modpackFile.value
					? formatMessage(messages.modpackUploadFailedTitle)
					: formatMessage(messages.installationFailedTitle),
			text:
				config.setupType.value === 'modpack' && config.modpackFile.value
					? formatMessage(messages.modpackUploadFailedText)
					: formatMessage(messages.installationFailedText),
			type: 'error',
		})
		config.loading.value = false
		uploading.value = false
	}
}

function createWorldContent(config: CreationFlowContextValue): Archon.Servers.v1.WorldContent {
	if (config.setupType.value === 'modpack' && config.modpackSelection.value) {
		return {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: config.modpackSelection.value.projectId,
				version_id: config.modpackSelection.value.versionId,
			},
		}
	}

	return createBareWorldContent(config)
}

function createBareWorldContent(config: CreationFlowContextValue): Archon.Servers.v1.WorldContent {
	const loader =
		config.setupType.value === 'vanilla' ? 'vanilla' : toApiLoader(config.selectedLoader.value)
	return {
		content_variant: 'bare',
		loader,
		version: loader === 'vanilla' ? '' : (config.selectedLoaderVersion.value ?? ''),
		game_version: config.selectedGameVersion.value ?? server.value?.mc_version ?? null,
	}
}

const steps = computed(() => [
	{
		icon: PackageIcon,
		title: formatMessage(messages.chooseWhatToPlayTitle),
		description: formatMessage(messages.chooseWhatToPlayDescription),
	},
	{
		icon: GlobeIcon,
		title: formatMessage(messages.configureInstanceTitle),
		description: formatMessage(messages.configureInstanceDescription),
	},
	{
		icon: UsersIcon,
		title: formatMessage(messages.inviteFriendsTitle),
		description: formatMessage(messages.inviteFriendsDescription),
	},
])
</script>
