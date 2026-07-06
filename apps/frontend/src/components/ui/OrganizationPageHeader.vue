<template>
	<PageHeader :title="organization.name" :summary="organization.description">
		<template #leading>
			<PageHeaderObjectAvatarLeading
				:src="organization.icon_url"
				:alt="organization.name"
				size="96px"
			/>
		</template>

		<template #badges>
			<PageHeaderBadgeItem :icon="OrganizationIcon" class="px-0 text-primary">
				{{ labels.organization }}
			</PageHeaderBadgeItem>
		</template>

		<template #metadata>
			<PageHeaderMetadata>
				<PageHeaderMetadataNumberItem
					:icon="UsersIcon"
					:value="membersCount"
					:label="labels.members"
				/>
				<PageHeaderMetadataNumberItem
					:icon="BoxIcon"
					:value="projectsCount"
					:label="labels.projects"
				/>
				<PageHeaderMetadataNumberItem
					:icon="DownloadIcon"
					:value="downloads"
					:label="labels.downloads"
					:tooltip="downloadsTooltip"
				/>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="canManage" size="large">
					<nuxt-link :to="`/organization/${organization.slug}/settings`">
						<SettingsIcon />
						{{ labels.manage }}
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="labels.moreOptions"
						:aria-label="labels.moreOptions"
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
	ButtonStyled,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	PageHeaderObjectAvatarLeading,
	TeleportOverflowMenu,
	type TeleportOverflowMenuItem,
} from '@modrinth/ui'
import { computed } from 'vue'

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
	downloadsTooltip?: string
	canManage?: boolean
	labels: {
		organization: string
		members: string
		projects: string
		downloads: string
		manage: string
		manageProjects: string
		moreOptions: string
		copyId: string
		copyPermalink: string
	}
}>()

const emit = defineEmits<{
	manageProjects: []
	copyId: []
	copyPermalink: []
}>()

const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'manage-projects',
		label: props.labels.manageProjects,
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
		label: props.labels.copyId,
		icon: ClipboardCopyIcon,
		action: () => emit('copyId'),
	},
	{
		id: 'copy-permalink',
		label: props.labels.copyPermalink,
		icon: ClipboardCopyIcon,
		action: () => emit('copyPermalink'),
	},
])
</script>
