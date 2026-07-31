<template>
	<PageHeader :title="organization.name" :summary="organization.description">
		<template #leading>
			<Avatar
				:src="organization.icon_url"
				:alt="organization.name"
				:tint-by="organization.id"
				size="96px"
			/>
		</template>

		<template #badges>
			<PageHeaderBadgeItem :icon="OrganizationIcon" class="px-0 text-primary">
				{{ formatMessage(messages.organizationLabel) }}
			</PageHeaderBadgeItem>
		</template>

		<template #metadata>
			<PageHeaderMetadata>
				<PageHeaderMetadataNumberItem
					:icon="UsersIcon"
					:value="membersCount"
					:label="formatMessage(messages.membersLabel)"
				/>
				<PageHeaderMetadataNumberItem
					:icon="BoxIcon"
					:value="projectsCount"
					:label="formatMessage(messages.projectsLabel)"
				/>
				<PageHeaderMetadataNumberItem
					:icon="DownloadIcon"
					:value="downloads"
					:label="formatMessage(messages.downloadsLabel)"
					:tooltip="formatNumber(downloads)"
				/>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="canManage" size="large">
					<nuxt-link :to="`/organization/${organization.slug}/settings`">
						<SettingsIcon />
						{{ formatMessage(messages.manage) }}
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="formatMessage(commonMessages.moreOptionsButton)"
						:aria-label="formatMessage(commonMessages.moreOptionsButton)"
					>
						<MoreVerticalIcon />
					</TeleportOverflowMenu>
				</ButtonStyled>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	MoreVerticalIcon,
	OrganizationIcon,
	SettingsIcon,
	UsersIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	TeleportOverflowMenu,
	type TeleportOverflowMenuItem,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

const messages = defineMessages({
	downloadsLabel: {
		id: 'organization.label.downloads',
		defaultMessage: 'downloads',
	},
	manage: {
		id: 'organization.button.manage',
		defaultMessage: 'Manage',
	},
	manageProjects: {
		id: 'organization.button.manage-projects',
		defaultMessage: 'Manage projects',
	},
	membersLabel: {
		id: 'organization.label.members',
		defaultMessage: 'members',
	},
	organizationLabel: {
		id: 'organization.label.organization',
		defaultMessage: 'Organization',
	},
	projectsLabel: {
		id: 'organization.label.projects',
		defaultMessage: 'projects',
	},
})

const props = defineProps<{
	organization: {
		id: string
		name: string
		slug: string
		description?: string | null
		icon_url?: string | null
	}
	membersCount: number
	projectsCount: number
	downloads: number
	canManage?: boolean
}>()

const emit = defineEmits<{
	manageProjects: []
	copyId: []
	copyPermalink: []
}>()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()

const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'manage-projects',
		label: formatMessage(messages.manageProjects),
		icon: BoxIcon,
		action: () => emit('manageProjects'),
		shown: props.canManage,
	},
	{
		divider: true,
		shown: props.canManage,
	},
	{
		id: 'copy-id',
		label: formatMessage(commonMessages.copyIdButton),
		icon: ClipboardCopyIcon,
		action: () => emit('copyId'),
	},
	{
		id: 'copy-permalink',
		label: formatMessage(commonMessages.copyPermalinkButton),
		icon: ClipboardCopyIcon,
		action: () => emit('copyPermalink'),
	},
])
</script>
