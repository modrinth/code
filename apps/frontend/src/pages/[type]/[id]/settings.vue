<script setup lang="ts">
import {
	AlignLeftIcon,
	BookTextIcon,
	ChartIcon,
	GlobeIcon,
	ImageIcon,
	InfoIcon,
	LinkIcon,
	TagsIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonProjectSettingsMessages,
	injectProjectPageContext,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useLocalStorage, useScroll } from '@vueuse/core'
import { computed } from 'vue'

import ModerationProjectNags from '~/components/ui/moderation/ModerationProjectNags.vue'
import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

const {
	projectV2: project,
	projectV3,
	versions,
	currentMember,
	setProcessing,
} = injectProjectPageContext()

const flags = useFeatureFlags()

const navItems = computed(() => {
	const base = `${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`

	const showEnvironment =
		projectV3.value?.project_types?.some((type) => ['mod', 'modpack'].includes(type)) &&
		isStaff(currentMember.value?.user)

	const items = [
		{
			link: `/${base}/settings`,
			label: formatMessage(commonProjectSettingsMessages.general),
			icon: InfoIcon,
		},
		flags.value.newProjectGeneralSettings
			? {
					link: `/${base}/settings/general`,
					label: formatMessage(commonProjectSettingsMessages.general),
					badge: formatMessage(commonMessages.newBadge),
					icon: InfoIcon,
				}
			: null,
		{
			link: `/${base}/settings/tags`,
			label: formatMessage(commonProjectSettingsMessages.tags),
			icon: TagsIcon,
		},
		{
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		{
			link: `/${base}/settings/versions`,
			label: formatMessage(commonProjectSettingsMessages.versions),
			icon: VersionIcon,
		},
		{
			link: `/${base}/settings/license`,
			label: formatMessage(commonProjectSettingsMessages.license),
			icon: BookTextIcon,
		},
		{
			link: `/${base}/settings/gallery`,
			label: formatMessage(commonProjectSettingsMessages.gallery),
			icon: ImageIcon,
		},
		{
			link: `/${base}/settings/links`,
			label: formatMessage(commonProjectSettingsMessages.links),
			icon: LinkIcon,
		},
		{
			link: `/${base}/settings/members`,
			label: formatMessage(commonProjectSettingsMessages.members),
			icon: UsersIcon,
		},
		{
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
		},
		{ type: 'heading', label: 'moderation', shown: showEnvironment },
		{
			link: `/${base}/settings/environment`,
			label: formatMessage(commonProjectSettingsMessages.environment),
			icon: GlobeIcon,
			shown: showEnvironment,
		},
	]
	return items.filter(Boolean) as any[]
})

const tags = useGeneratedState()
const route = useRoute()
const collapsedChecklist = useLocalStorage(`project-checklist-collapsed-${project.value.id}`, false)

// To persist scroll position through settings pages
// This scroll code is jank asf, if anyone has a better way please do suggest it
const scroll = useScroll(window)
watch(route, () => {
	const scrollY = scroll.y.value
	setTimeout(() => window.scrollTo(0, scrollY), 10)
})
</script>

<template>
	<div class="mb-8 flex w-full flex-col gap-4">
		<ModerationProjectNags
			v-if="
				(currentMember && project.status === 'draft') ||
				tags.rejectedStatuses.includes(project.status)
			"
			:project="project"
			:versions="versions"
			:current-member="currentMember"
			:collapsed="collapsedChecklist"
			:route-name="route.name"
			:tags="tags"
			@toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
			@set-processing="setProcessing"
		/>
		<div class="experimental-styles-within grid gap-4 lg:grid-cols-[1fr_3fr]">
			<div>
				<NavStack :items="navItems" />
			</div>
			<div class="min-w-0">
				<NuxtPage />
			</div>
		</div>
	</div>
</template>
