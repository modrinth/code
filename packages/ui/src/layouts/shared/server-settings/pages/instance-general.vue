<template>
	<div class="flex h-full w-full flex-col gap-6">
		<Teleport to="body">
			<div class="relative z-[100]">
				<ServerSetupModal
					ref="setupModal"
					@reinstall="onResetEverything"
					@browse-modpacks="onBrowseModpacks"
				/>
			</div>
		</Teleport>

		<div class="flex max-w-[496px] flex-col gap-2.5">
			<label for="instance-name-field" class="font-semibold text-contrast">
				{{ formatMessage(messages.instanceNameLabel) }}
			</label>
			<div v-tooltip="formatMessage(messages.notImplemented)" class="max-w-[400px]">
				<StyledInput
					id="instance-name-field"
					:model-value="instanceName"
					disabled
					wrapper-class="w-full"
				/>
			</div>
			<span class="text-primary">{{ formatMessage(messages.instanceNameDescription) }}</span>
		</div>

		<div class="flex flex-col gap-2.5">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.dangerZoneLabel) }}
			</span>
			<div class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-5 p-4">
				<div class="flex flex-col items-start gap-2.5">
					<div v-tooltip="formatMessage(messages.notImplemented)">
						<ButtonStyled color="red">
							<button class="!shadow-none" disabled>
								<RotateCounterClockwiseIcon class="size-5" />
								{{ formatMessage(messages.resetWorldFilesButton) }}
							</button>
						</ButtonStyled>
					</div>
					<span class="text-primary">
						{{ formatMessage(messages.resetWorldFilesDescription) }}
					</span>
				</div>

				<div class="flex flex-col items-start gap-2.5">
					<ButtonStyled color="red">
						<button class="!shadow-none" :disabled="isResetDisabled" @click="setupModal?.show()">
							<TrashIcon class="size-5" />
							{{ formatMessage(messages.resetEverythingButton) }}
						</button>
					</ButtonStyled>
					<span class="text-primary">
						{{ formatMessage(messages.resetEverythingDescription) }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { RotateCounterClockwiseIcon, TrashIcon } from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { ButtonStyled, StyledInput } from '#ui/components'
import ServerSetupModal from '#ui/components/servers/ServerSetupModal.vue'
import { useModrinthServersConsole } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectServerSettings } from '#ui/layouts/shared/server-settings'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { server, serverId, worldId, isSyncingContent, busyReasons } = injectModrinthServerContext()
const serverSettings = injectServerSettings()
const queryClient = useQueryClient()
const modrinthServersConsole = useModrinthServersConsole()

const setupModal = ref<InstanceType<typeof ServerSetupModal>>()

const messages = defineMessages({
	instanceNameLabel: {
		id: 'server.instance-settings.general.instance-name.label',
		defaultMessage: 'Instance name',
	},
	instanceNameDescription: {
		id: 'server.instance-settings.general.instance-name.description',
		defaultMessage: 'This name is only visible on Modrinth.',
	},
	instanceFallbackName: {
		id: 'server.instance-settings.general.instance-name.fallback',
		defaultMessage: 'Instance',
	},
	dangerZoneLabel: {
		id: 'server.instance-settings.general.danger-zone.label',
		defaultMessage: 'Danger zone',
	},
	resetWorldFilesButton: {
		id: 'server.instance-settings.general.reset-world-files.button',
		defaultMessage: 'Reset world data',
	},
	resetWorldFilesDescription: {
		id: 'server.instance-settings.general.reset-world-files.description',
		defaultMessage:
			'Delete the current world data and generate a new one. Your content and files will stay the same.',
	},
	resetEverythingButton: {
		id: 'server.instance-settings.general.reset-everything.button',
		defaultMessage: 'Reset everything',
	},
	resetEverythingDescription: {
		id: 'server.instance-settings.general.reset-everything.description',
		defaultMessage:
			'Reset your instance completely. This removes world data, content, and any configuration. A backup of the previous instance will remain available.',
	},
	notImplemented: {
		id: 'server.instance-settings.general.not-implemented',
		defaultMessage: 'Not yet implemented',
	},
})

const serverFullQuery = useQuery({
	queryKey: ['servers', 'v1', 'detail', serverId],
	queryFn: () => client.archon.servers_v1.get(serverId),
	staleTime: 30_000,
})

const currentWorld = computed(() => {
	const id = worldId.value
	if (!id) return null
	return serverFullQuery.data.value?.worlds.find((world) => world.id === id) ?? null
})

const instanceName = computed(
	() => currentWorld.value?.name ?? formatMessage(messages.instanceFallbackName),
)

const isResetDisabled = computed(
	() =>
		!worldId.value ||
		server.value?.status === 'installing' ||
		isSyncingContent.value ||
		busyReasons.value.length > 0,
)

function onResetEverything() {
	modrinthServersConsole.clear()
	queryClient.removeQueries({ queryKey: ['servers', 'ws-state', serverId] })
	void Promise.all([
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
		worldId.value
			? queryClient.invalidateQueries({
					queryKey: ['content', 'list', 'v1', serverId, worldId.value],
				})
			: Promise.resolve(),
	])
	serverSettings.closeModal?.()
}

function onBrowseModpacks() {
	serverSettings.browseModpacks({
		serverId,
		worldId: worldId.value,
		from: 'reset-server',
	})
}
</script>
