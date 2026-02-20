<template>
	<NewModal ref="modal" :header="formatMessage(messages.installToPlay)" :closable="true">
		<div v-if="requiredContentProject" class="flex flex-col gap-6 max-w-[500px]">
			<Admonition type="info" :header="formatMessage(messages.contentRequired)">
				{{ formatMessage(messages.serverRequiresMods) }}
			</Admonition>

			<div v-if="sharedBy?.name" class="flex items-center gap-2 text-sm text-secondary">
				<Avatar
					v-if="sharedBy?.icon_url"
					:src="sharedBy.icon_url"
					:alt="sharedBy.name"
					size="24px"
				/>
				<span>
					<IntlFormatted :message-id="messages.sharedByToday">
						<template #~name>
							<span class="font-semibold text-contrast">{{ sharedBy.name }}</span>
						</template>
					</IntlFormatted>
				</span>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{
					formatMessage(messages.requiredModpack)
				}}</span>
				<div class="flex items-center gap-3 rounded-xl bg-surface-2 p-3">
					<Avatar
						:src="requiredContentProject.icon_url"
						:alt="requiredContentProject.title"
						size="48px"
					/>
					<div class="flex flex-col gap-0.5">
						<span class="font-semibold text-contrast">{{ requiredContentProject.title }}</span>
						<span class="text-sm text-secondary">
							{{ loaderDisplay }} {{ requiredContentProject.game_versions?.[0] }}
							<template v-if="modCount">
								· {{ formatMessage(messages.modCount, { count: modCount }) }}
							</template>
						</span>
					</div>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="handleDecline">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleAccept">
						<DownloadIcon />
						{{ formatMessage(messages.installButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	formatLoader,
	IntlFormatted,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { get_organization, get_project, get_team, get_version } from '@/helpers/cache.js'
import { install } from '@/store/install.js'
import type { Labrinth } from '@modrinth/api-client'

const modal = ref<InstanceType<typeof NewModal>>()
const modpackVersionId = ref<string | null>(null)
const modpackVersion = ref<any>(null)
const project = ref<any>(null)
const requiredContentProject = ref<any>(null)
const organization = ref<any>(null)
const teamMembers = ref<any[]>([])
const onInstallComplete = ref<() => void>(() => {})
const { formatMessage } = useVIntl()

const sharedBy = computed(() => {
	if (organization.value) {
		return {
			name: organization.value.name,
			icon_url: organization.value.icon_url,
		}
	}
	if (teamMembers.value?.length) {
		const owner = teamMembers.value.find((member: { is_owner: boolean }) => member.is_owner)
		if (owner) {
			return {
				name: owner.user.username,
				icon_url: owner.user.avatar_url,
			}
		}
	}
	return null
})

const loaderDisplay = computed(() => {
	const loader = requiredContentProject.value?.loaders?.[0]
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})

const modCount = computed(() => modpackVersion.value?.dependencies?.length)

async function fetchData(versionId: string) {
	// cache is making version null for some reason so bypassing for now
	modpackVersion.value = await get_version(versionId, 'bypass')

	if (modpackVersion.value?.project_id) {
		requiredContentProject.value = await get_project(modpackVersion.value.project_id, 'bypass')
	}

	if (project.value?.organization) {
		organization.value = await get_organization(project.value.organization, 'bypass')
	} else if (project.value?.team_id) {
		teamMembers.value = await get_team(project.value.team_id, 'bypass')
	}
}

async function handleAccept() {
	hide()
	try {
		await install(
			modpackVersion.value?.project_id,
			modpackVersionId.value,
			null,
			'ProjectPageInstallToPlayModal',
		)
		onInstallComplete.value()
	} catch (error) {
		console.error('Failed to install project from InstallToPlayModal:', error)
	}
}

function handleDecline() {
	hide()
}

async function show(
	projectVal: Labrinth.Projects.v3.Project,
	modpackVersionIdVal: string | null = null,
	callback: () => void = () => {},
	e?: MouseEvent,
) {
	project.value = projectVal
	modpackVersionId.value = modpackVersionIdVal
	modpackVersion.value = null
	requiredContentProject.value = null
	organization.value = null
	teamMembers.value = []
	onInstallComplete.value = callback

	if (modpackVersionIdVal) await fetchData(modpackVersionIdVal)

	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

const messages = defineMessages({
	installToPlay: {
		id: 'app.modal.install-to-play.header',
		defaultMessage: 'Install to play',
	},
	sharedServerInstance: {
		id: 'app.modal.install-to-play.shared-server-instance',
		defaultMessage: 'Shared server instance',
	},
	contentRequired: {
		id: 'app.modal.install-to-play.content-required',
		defaultMessage: 'Content required',
	},
	serverRequiresMods: {
		id: 'app.modal.install-to-play.server-requires-mods',
		defaultMessage:
			'This server requires mods to play. Click Install to set up the required files from Modrinth, then launch directly into the server.',
	},
	requiredModpack: {
		id: 'app.modal.install-to-play.required-modpack',
		defaultMessage: 'Required modpack',
	},
	sharedByToday: {
		id: 'app.modal.install-to-play.shared-by-today',
		defaultMessage: '{name} shared this instance with you today.',
	},
	sharedInstance: {
		id: 'app.modal.install-to-play.shared-instance',
		defaultMessage: 'Shared instance',
	},
	modCount: {
		id: 'app.modal.install-to-play.mod-count',
		defaultMessage: '{count, plural, one {# mod} other {# mods}}',
	},
	installButton: {
		id: 'app.modal.install-to-play.install-button',
		defaultMessage: 'Install',
	},
})

defineExpose({ show, hide })
</script>
