<script setup lang="ts">
import {
	AlignLeftIcon,
	ChartIcon,
	GlobeIcon,
	ImageIcon,
	InfoIcon,
	IssuesIcon,
	LinkIcon,
	ServerIcon,
	SignatureIcon,
	TagsIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	commonProjectSettingsMessages,
	defineMessages,
	injectProjectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { computed, useId } from 'vue'

import NavStack, { type NavStackEntry } from '~/components/ui/NavStack.vue'

import { getSettingsIssueCounts } from './destinations'

const messages = defineMessages({
	details: { id: 'project.settings.navigation.details', defaultMessage: 'Details' },
	content: { id: 'project.settings.navigation.content', defaultMessage: 'Content' },
	manage: { id: 'project.settings.navigation.manage', defaultMessage: 'Manage' },
	showMemberUi: {
		id: 'project.settings.navigation.show-member-ui',
		defaultMessage: 'Show member UI',
	},
})
const { formatMessage } = useVIntl()
const {
	projectV2: project,
	projectV3,
	currentMember,
	projectValidation,
	thread,
} = injectProjectPageContext()
const flags = useFeatureFlags()

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const isStaffMember = computed(() => isStaff(currentMember.value?.user))
const settingsIssueCounts = computed(() =>
	getSettingsIssueCounts(
		project.value.project_type,
		projectValidation.value?.nags ?? [],
		thread.value?.issues ?? [],
	),
)

const navItems = computed(() => {
	const base = `${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`

	const showEnvironment =
		!isServerProject.value &&
		isStaffMember.value &&
		!flags.value.showModeratorProjectMemberUi &&
		projectV3.value?.project_types?.some((type) => ['mod', 'modpack'].includes(type))

	const hasPermissionsPage = projectV3.value?.project_types?.includes('modpack')

	const items: (NavStackEntry | false | undefined)[] = [
		{ type: 'heading', label: formatMessage(messages.details) },
		{
			link: `/${base}/settings`,
			label: formatMessage(commonProjectSettingsMessages.general),
			icon: InfoIcon,
		},
		isServerProject.value && {
			link: `/${base}/settings/server`,
			label: formatMessage(commonProjectSettingsMessages.server),
			icon: ServerIcon,
		},
		{
			link: `/${base}/settings/disclosures`,
			label: formatMessage(commonProjectSettingsMessages.disclosures),
			icon: IssuesIcon,
		},
		{
			link: `/${base}/settings/tags`,
			label: formatMessage(commonProjectSettingsMessages.tags),
			icon: TagsIcon,
		},
		{
			link: `/${base}/settings/links`,
			label: formatMessage(commonProjectSettingsMessages.links),
			icon: LinkIcon,
		},
		{ type: 'heading', label: formatMessage(messages.content) },
		!isServerProject.value && {
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/versions`,
			label: formatMessage(commonProjectSettingsMessages.versions),
			icon: VersionIcon,
		},
		hasPermissionsPage && {
			link: `/${base}/settings/permissions`,
			label: formatMessage(commonProjectSettingsMessages.permissions),
			icon: SignatureIcon,
		},
		isServerProject.value && {
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		{
			link: `/${base}/settings/gallery`,
			label: formatMessage(commonProjectSettingsMessages.gallery),
			icon: ImageIcon,
		},
		{ type: 'heading', label: formatMessage(messages.manage) },
		{
			link: `/${base}/settings/members`,
			label: formatMessage(commonProjectSettingsMessages.members),
			icon: UsersIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
		},
		showEnvironment && {
			type: 'heading',
			label: 'moderation',
		},
		showEnvironment && {
			link: `/${base}/settings/environment`,
			label: formatMessage(commonProjectSettingsMessages.environment),
			icon: GlobeIcon,
		},
	]
	return items
		.filter((item) => !!item)
		.map((item) => {
			if (!('link' in item) || !item.link) return item
			const issues = settingsIssueCounts.value.get(item.link.slice(`/${base}/`.length))
			return {
				...item,
				issueCount: issues?.count,
				hasValidationErrors: issues?.hasValidationErrors,
			}
		})
})

const moderatorSeeUserUiId = useId()
const moderatorSeeUserUi = computed<boolean>({
	get() {
		return flags.value.showModeratorProjectMemberUi
	},
	set(value: boolean) {
		flags.value.showModeratorProjectMemberUi = value
		saveFeatureFlags()
	},
})
</script>

<template>
	<NavStack :items="navItems" />
	<div v-if="isStaffMember" class="mt-4 flex items-center gap-2">
		<Toggle :id="moderatorSeeUserUiId" v-model="moderatorSeeUserUi" small />
		<label :for="moderatorSeeUserUiId">
			{{ formatMessage(messages.showMemberUi) }}
		</label>
	</div>
</template>
