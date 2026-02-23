<template>
	<div class="mx-auto flex w-fit flex-col items-start gap-4 mt-4">
		<div class="flex flex-col gap-2 w-full">
			<h2 class="m-0 text-2xl font-semibold text-contrast">Welcome to Modrinth</h2>
			<p class="m-0 text-base text-secondary">
				Your server is ready. Here's what you need to do to start playing!
			</p>
		</div>

		<div class="flex flex-col gap-4">
			<span class="text-base font-medium text-secondary"> Setup your server (~2mins) </span>

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
			<ButtonStyled color="brand" size="large">
				<button class="ml-auto" @click="openModal">Continue <RightArrowIcon /></button>
			</ButtonStyled>
		</div>

		<CreationFlowModal
			ref="modalRef"
			type="server-onboarding"
			:show-snapshot-toggle="true"
			@hide="() => {}"
			@browse-modpacks="() => {}"
			@create="onCreate"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { GlobeIcon, PackageIcon, RightArrowIcon, UsersIcon } from '@modrinth/assets'
import { ButtonStyled, injectModrinthClient } from '@modrinth/ui'
import { onBeforeUnmount, ref } from 'vue'

import type { CreationFlowContextValue } from '../../../../components'
import { CreationFlowModal } from '../../../../components'
import { injectModrinthServerContext } from '../../../../providers'

const client = injectModrinthClient()
const { serverId, worldId, server } = injectModrinthServerContext()

const modalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)

const openModal = () => modalRef.value?.show()

onBeforeUnmount(() => modalRef.value?.hide())

/** Map UI loader names to API Modloader values */
function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

const onCreate = async (config: CreationFlowContextValue) => {
	let request: Archon.Content.v1.InstallWorldContent

	if (config.setupType.value === 'modpack' && config.modpackSelection.value) {
		request = {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: config.modpackSelection.value.projectId,
				version_id: config.modpackSelection.value.versionId,
			},
			soft_override: false,
		}
	} else {
		const loader = config.selectedLoader.value
		request = {
			content_variant: 'bare',
			loader: loader ? toApiLoader(loader) : 'vanilla',
			version: config.selectedLoaderVersion.value ?? '',
			game_version: config.selectedGameVersion.value ?? undefined,
			soft_override: false,
		}
	}

	// TODO: POST server.properties fields (worldName, gamemode, difficulty, seed, etc.) once the endpoint is available

	try {
		await client.archon.content_v1.installContent(serverId, request, worldId.value ?? undefined)
		server.value.status = 'installing'
	} catch {
		config.loading.value = false
	}
}

const steps = [
	{
		icon: GlobeIcon,
		title: 'Create your first world',
		description:
			'A world is a switchable copy of your server that lets you play different mods without resetting anything.',
	},
	{
		icon: PackageIcon,
		title: 'Choose what to play',
		description:
			'Pick your favorite modpack from Modrinth, or choose a loader and add the mods you want.',
	},
	// TODO: Enable when sharing is impl
	{
		icon: UsersIcon,
		title: 'Invite your friends',
		description:
			'Share your server with friends so they can join and automatically download everything they need!',
	},
]
</script>
