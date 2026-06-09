<template>
	<div class="normal-page !mt-8">
		<div class="normal-page__sidebar">
			<NavStack
				:items="[
					{ type: 'heading', label: formatMessage(messages.dashboard) },
					{ link: '/dashboard', label: formatMessage(messages.overview), icon: DashboardIcon },
					{
						link: '/dashboard/notifications',
						label: formatMessage(messages.notifications),
						icon: NotificationsIcon,
					},
					{
						link: '/dashboard/reports',
						label: formatMessage(messages.activeReports),
						icon: ReportIcon,
					},
					{
						link: '/dashboard/collections',
						label: formatMessage(commonMessages.collectionsLabel),
						icon: LibraryIcon,
					},
					{ type: 'heading', label: formatMessage(messages.creators) },
					{ link: '/dashboard/projects', label: formatMessage(messages.projects), icon: ListIcon },
					{
						link: '/dashboard/organizations',
						label: formatMessage(messages.organizations),
						icon: OrganizationIcon,
					},
					{
						link: '/dashboard/analytics',
						label: formatMessage(messages.analytics),
						icon: ChartIcon,
					},
					{
						link: '/dashboard/affiliate-links',
						label: formatMessage(commonMessages.affiliateLinksButton),
						icon: AffiliateIcon,
						shown: !!isAffiliate,
					},
					{
						link: '/dashboard/revenue',
						label: formatMessage(messages.revenue),
						icon: CurrencyIcon,
						matchNested: true,
					},
				]"
			/>
		</div>
		<div class="normal-page__content mt-4 lg:!mt-0">
			<Admonition
				v-if="showDiscordRoleBanner"
				class="mb-3"
				type="info"
				:header="formatMessage(messages.discordRoleBannerTitle)"
				show-actions-underneath
				dismissible
				@dismiss="dismissDiscordRoleBanner"
			>
				<div class="text-primary">
					{{
						formatMessage(messages.discordRoleBannerBody, {
							roles: eligibleDiscordRolesLabel,
						})
					}}
				</div>
				<template #actions>
					<ButtonStyled color="blue">
						<NuxtLink to="/discord/link" class="w-fit !px-4">
							<ExternalIcon />
							{{ formatMessage(messages.discordRoleBannerCta) }}
						</NuxtLink>
					</ButtonStyled>
				</template>
			</Admonition>
			<NuxtPage :route="route" />
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AffiliateIcon,
	BellIcon as NotificationsIcon,
	ChartIcon,
	CurrencyIcon,
	DashboardIcon,
	ExternalIcon,
	LibraryIcon,
	ListIcon,
	OrganizationIcon,
	ReportIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'
import { UserBadge } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { useLocalStorage } from '@vueuse/core'

import NavStack from '~/components/ui/NavStack.vue'

const auth = (await useAuth()) as Ref<{ user: Labrinth.Users.v3.User | null }>
const client = injectModrinthClient()
const dismissedDiscordRoleBannerUsers = useLocalStorage<string[]>(
	'dashboard-discord-role-banner-dismissed-users',
	[],
)

const isAffiliate = computed(() => {
	return auth.value.user && auth.value.user.badges & UserBadge.AFFILIATE
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	dashboard: {
		id: 'dashboard.sidebar.label.dashboard',
		defaultMessage: 'Dashboard',
	},
	overview: {
		id: 'dashboard.sidebar.label.overview',
		defaultMessage: 'Overview',
	},
	notifications: {
		id: 'dashboard.sidebar.label.notifications',
		defaultMessage: 'Notifications',
	},
	activeReports: {
		id: 'dashboard.sidebar.label.activeReports',
		defaultMessage: 'Active reports',
	},
	creators: {
		id: 'dashboard.sidebar.label.creators',
		defaultMessage: 'Creators',
	},
	projects: {
		id: 'dashboard.sidebar.label.projects',
		defaultMessage: 'Projects',
	},
	organizations: {
		id: 'dashboard.sidebar.label.organizations',
		defaultMessage: 'Organizations',
	},
	analytics: {
		id: 'dashboard.sidebar.label.analytics',
		defaultMessage: 'Analytics',
	},
	revenue: {
		id: 'dashboard.sidebar.label.revenue',
		defaultMessage: 'Revenue',
	},
	discordRoleBannerTitle: {
		id: 'dashboard.discord-roles.banner.title',
		defaultMessage: 'Claim your Discord roles',
	},
	discordRoleBannerBody: {
		id: 'dashboard.discord-roles.banner.body',
		defaultMessage:
			"You're eligible for {roles}. Link your Discord account through Modrinth and we'll sync them automatically.",
	},
	discordRoleBannerCta: {
		id: 'dashboard.discord-roles.banner.cta',
		defaultMessage: 'Link Discord',
	},
	discordRolePride: {
		id: 'dashboard.discord-roles.role.pride',
		defaultMessage: 'Pride 2026',
	},
	discordRoleCreator: {
		id: 'dashboard.discord-roles.role.creator',
		defaultMessage: 'Creator',
	},
	discordRoleBigCreator: {
		id: 'dashboard.discord-roles.role.big-creator',
		defaultMessage: '1M+ Downloads',
	},
})

definePageMeta({
	middleware: 'auth',
})

useSeoMeta({
	robots: 'noindex',
})

const route = useNativeRoute()

const hasLinkedDiscordAccount = computed(() =>
	auth.value.user?.auth_providers?.includes('discord') === true,
)

const { data: projects } = useQuery({
	queryKey: computed(() => ['dashboard-discord-role-eligibility', auth.value.user?.id, 'projects']),
	queryFn: () => {
		const userId = auth.value.user?.id
		if (!userId) return []

		return client.labrinth.users_v2.getProjects(userId)
	},
	enabled: computed(() => !!auth.value.user?.id && !hasLinkedDiscordAccount.value),
})

const totalProjectDownloads = computed(() =>
	(projects.value ?? []).reduce((total, project) => total + (project.downloads ?? 0), 0),
)

const eligibleDiscordRoles = computed(() => {
	const roles = []

	if (auth.value.user?.campaigns?.pride_26?.has_badge === true) {
		roles.push(formatMessage(messages.discordRolePride))
	}

	if (totalProjectDownloads.value >= 20_000) {
		roles.push(formatMessage(messages.discordRoleCreator))
	}

	if (totalProjectDownloads.value >= 1_000_000) {
		roles.push(formatMessage(messages.discordRoleBigCreator))
	}

	return roles
})

const roleListFormatter = new Intl.ListFormat(undefined, {
	style: 'long',
	type: 'conjunction',
})

const eligibleDiscordRolesLabel = computed(() =>
	roleListFormatter.format(eligibleDiscordRoles.value),
)

const hasDismissedDiscordRoleBanner = computed(() =>
	dismissedDiscordRoleBannerUsers.value.includes(auth.value.user?.id ?? ''),
)
const showDiscordRoleBanner = computed(
	() =>
		eligibleDiscordRoles.value.length > 0 &&
		!hasLinkedDiscordAccount.value &&
		!hasDismissedDiscordRoleBanner.value,
)

function dismissDiscordRoleBanner() {
	const userId = auth.value.user?.id
	if (!userId || dismissedDiscordRoleBannerUsers.value.includes(userId)) return

	dismissedDiscordRoleBannerUsers.value = [...dismissedDiscordRoleBannerUsers.value, userId]
}
</script>
