<script setup lang="ts">
import {
	AlignLeftIcon,
	BookTextIcon,
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
import { getProjectNagCategory, normalizeProjectNagKind, toProjectNag } from '@modrinth/moderation'
import {
	issueLocationRegistry,
	readIssueLocations,
} from '@modrinth/moderation/src/data/issues/component-builders/locations'
import {
	commonMessages,
	commonProjectSettingsMessages,
	injectProjectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useScroll } from '@vueuse/core'
import { computed, ref } from 'vue'

import ProjectModerationDrawer from '~/components/ui/moderation/ProjectModerationDrawer.vue'
import NavStack from '~/components/ui/NavStack.vue'
import { threadIssueField } from '~/components/ui/project-issue-card/issue-targets'
import { isThreadIssueVerified, threadIssueVerdict } from '~/helpers/thread-issues'

const { formatMessage } = useVIntl()

const {
	projectV2: project,
	projectV3,
	currentMember,
	projectValidation,
	thread,
} = injectProjectPageContext()

const drawerPinned = ref(false)
const flags = useFeatureFlags()

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const isStaffMember = computed(() => isStaff(currentMember.value?.user))

const settingsIssueCounts = computed(() => {
	const counts = new Map<string, { count: number; hasValidationErrors: boolean }>()
	const increment = (path: string, hasValidationErrors = false) => {
		const previous = counts.get(path)
		counts.set(path, {
			count: (previous?.count ?? 0) + 1,
			hasValidationErrors: hasValidationErrors || (previous?.hasValidationErrors ?? false),
		})
	}

	for (const nag of projectValidation.value?.nags ?? []) {
		if (nag.severity === 'suggestion' || !normalizeProjectNagKind(nag.kind)) continue
		const path = toProjectNag(nag, project.value.project_type).link?.path
		if (path?.startsWith('settings')) {
			increment(
				path,
				nag.severity === 'required' && getProjectNagCategory(nag.kind) === 'validation',
			)
		}
	}

	for (const issue of thread.value?.issues ?? []) {
		if (isThreadIssueVerified(issue) || threadIssueVerdict(issue) !== 'open') continue
		const fields = [
			...readIssueLocations(issue.why?.locations).map((location) => location.field),
			...issue.facets
				.filter((facet) => facet.verdict === 'open' && !facet.moderator_verified)
				.map((facet) => threadIssueField(facet.what)),
		]
		const paths = new Set(
			fields.flatMap((field) => {
				if (!field) return []
				const area = issueLocationRegistry[field].area
				return [area ? `settings/${area}` : 'settings']
			}),
		)
		for (const path of paths) increment(path)
	}

	return counts
})

const navItems = computed(() => {
	const base = `${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`

	const showEnvironment =
		projectV3.value?.project_types?.some((type) => ['mod', 'modpack'].includes(type)) &&
		isStaffMember.value

	const hasPermissionsPage = computed(() =>
		projectV3.value?.project_types?.some((type) => ['modpack'].includes(type)),
	)

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
		hasPermissionsPage.value && {
			link: `/${base}/settings/permissions`,
			label: formatMessage(commonProjectSettingsMessages.permissions),
			icon: SignatureIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/license`,
			label: formatMessage(commonProjectSettingsMessages.license),
			icon: BookTextIcon,
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
		!isServerProject.value && {
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
		},
		!isServerProject.value && { type: 'heading', label: 'moderation', shown: showEnvironment },
		!isServerProject.value && {
			link: `/${base}/settings/environment`,
			label: formatMessage(commonProjectSettingsMessages.environment),
			icon: GlobeIcon,
			shown: showEnvironment,
		},
	]
	return (items.filter(Boolean) as any[]).map((item) => {
		const issues = settingsIssueCounts.value.get(
			item.link === `/${base}/settings/general`
				? 'settings'
				: item.link?.slice(`/${base}/`.length),
		)
		return {
			...item,
			issueCount: issues?.count,
			hasValidationErrors: issues?.hasValidationErrors,
		}
	})
})

const route = useRoute()

// To persist scroll position through settings pages
// This scroll code is jank asf, if anyone has a better way please do suggest it
const scroll = useScroll(window)
watch(route, () => {
	const scrollY = scroll.y.value
	setTimeout(() => window.scrollTo(0, scrollY), 10)
})

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
	<div
		class="project-settings mb-8 flex w-full flex-col gap-4"
		:class="{ 'settings-with-review': drawerPinned }"
	>
		<ProjectModerationDrawer v-if="currentMember && projectV3" @pinned="drawerPinned = $event" />
		<div class="grid gap-6 lg:grid-cols-[1fr_3fr]">
			<div class="lg:sticky lg:top-6 lg:self-start">
				<NavStack :items="navItems" />
				<div v-if="isStaffMember" class="mt-4 flex items-center gap-2">
					<Toggle id="moderator-see-user-ui-toggle" v-model="moderatorSeeUserUi" small />
					<label for="moderator-see-user-ui-toggle"> Show member UI </label>
				</div>
			</div>
			<div class="min-w-0">
				<NuxtPage />
			</div>
		</div>
	</div>
</template>

<style scoped>
.project-settings :deep(.settings-section-title) {
	margin: 0 0 12px;
	font-size: 20px;
	line-height: 28px;
	font-weight: 600;
	color: var(--color-contrast);
}

.project-settings :deep(.settings-content-card) {
	margin-top: 0;
	padding: 24px;
	border: 1px solid var(--surface-5);
	border-radius: 16px;
	background: var(--surface-3);
}

@media (max-width: 640px) {
	.project-settings :deep(.settings-content-card) {
		padding: 16px;
	}
}

.project-settings {
	box-sizing: border-box;
	transition: padding-right 240ms cubic-bezier(0.16, 1, 0.3, 1);
}

@media (prefers-reduced-motion: reduce) {
	.project-settings {
		transition: none;
	}
}

@media (min-width: 1280px) {
	.settings-with-review {
		padding-right: var(--project-settings-panel-offset, 0px);
	}
}
</style>
