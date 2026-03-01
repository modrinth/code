<template>
	<NewModal ref="modal" :header="formatMessage(messages.installToPlay)" :closable="true">
		<div class="flex flex-col gap-6 max-w-[500px]">
			<Admonition type="info" :header="formatMessage(messages.sharedServerInstance)">
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
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.sharedInstance) }}
				</span>
				<div class="flex items-center gap-3 rounded-xl bg-surface-4 p-3">
					<Avatar :src="project.icon_url" :alt="project.title" size="48px" />
					<div class="flex flex-col gap-0.5">
						<span class="font-semibold text-contrast">{{ project.title }}</span>
						<span class="text-sm text-secondary">
							{{ loaderDisplay }} {{ project.game_versions?.[0] }}
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
import type { Labrinth } from '@modrinth/api-client'
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
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { get_organization, get_team, get_version } from '@/helpers/cache.js'
import { install } from '@/composables/content-install'

const props = defineProps<{
	project: Labrinth.Projects.v2.Project
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const { formatMessage } = useVIntl()

const { data: organization } = useQuery({
	queryKey: computed(() => ['organization', props.project.organization]),
	queryFn: () => get_organization(props.project.organization!, 'must_revalidate'),
	enabled: computed(() => !!props.project.organization),
})

const { data: teamMembers } = useQuery({
	queryKey: computed(() => ['team', props.project.team]),
	queryFn: () => get_team(props.project.team, 'must_revalidate'),
	enabled: computed(() => !!props.project.team && !props.project.organization),
})

const sharedBy = computed(() => {
	if (organization.value) {
		return {
			name: organization.value.name,
			icon_url: organization.value.icon_url,
		}
	}
	if (teamMembers.value) {
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
	const loader = props.project.loaders?.[0]
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})

// Fetch the most recent version to get mod count from dependencies
const latestVersionId = computed(() => props.project.versions?.[0] ?? null)
const { data: latestVersion } = useQuery({
	queryKey: computed(() => ['version', latestVersionId.value]),
	queryFn: () => get_version(latestVersionId.value, 'must_revalidate'),
	enabled: computed(() => !!latestVersionId.value),
})
const modCount = computed(() => latestVersion.value?.dependencies?.length)

async function handleAccept() {
	hide()
	try {
		await install(props.project.id, null, null, 'ProjectPageInstallToPlayModal')
	} catch (error) {
		console.error('Failed to install project from InstallToPlayModal:', error)
	}
}

function handleDecline() {
	hide()
}

function show(e?: MouseEvent) {
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
	serverRequiresMods: {
		id: 'app.modal.install-to-play.server-requires-mods',
		defaultMessage:
			'This server requires mods to play. Click install to set up the required files from Modrinth.',
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
