<template>
	<div class="pointer-events-none fixed inset-0 z-[-1]">
		<div id="fixed-background-teleport" class="relative"></div>
	</div>
	<div class="pointer-events-none absolute inset-0 z-[-1]">
		<div id="absolute-background-teleport" class="relative"></div>
	</div>
	<div class="pointer-events-none absolute inset-0 z-50">
		<div
			class="over-the-top-random-animation"
			:style="{ '--_r-count': rCount }"
			:class="{ threshold: rCount > 20, 'rings-expand': rCount >= 40 }"
		>
			<div>
				<div
					class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
				></div>
				<div
					class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
				></div>
				<div
					class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight text-9xl font-extrabold text-contrast"
				>
					?
				</div>
			</div>
		</div>
	</div>
	<div
		ref="main_page"
		class="layout"
		:class="{
			'expanded-mobile-nav': isBrowseMenuOpen,
			'modrinth-parent__no-modal-blurs': !cosmetics.advancedRendering,
		}"
	>
		<RussiaBanner v-if="isRussia" />
		<TaxIdMismatchBanner v-if="showTinMismatchBanner" />
		<TaxComplianceBanner v-if="showTaxComplianceBanner" />
		<VerifyEmailBanner
			v-if="auth.user && !auth.user.email_verified && route.path !== '/auth/verify-email'"
			:has-email="!!auth?.user?.email"
		/>
		<SubscriptionPaymentFailedBanner
			v-if="
				user.subscriptions.some((x) => x.status === 'payment-failed') &&
				route.path !== '/settings/billing'
			"
		/>
		<PreviewBanner v-if="config.public.buildEnv === 'production' && config.public.preview" />
		<StagingBanner v-if="config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com')" />
		<GeneratedStateErrorsBanner
			:errors="generatedStateErrors"
			:api-url="config.public.apiBaseUrl"
		/>
		<header
			class="experimental-styles-within desktop-only relative z-[5] mx-auto flex w-[1200px] max-w-[1200px] items-center pr-4"
		>
			<div class="px-4 py-2">
				<NuxtLink to="/" :aria-label="formatMessage(messages.modrinthHomePage)" class="flex h-12">
					<img src="/modrinth-vista-dark.png" alt="Modrinth logo" />
				</NuxtLink>
			</div>
			<NuxtLink
				to="/discover/mods"
				class="px-2 py-2"
				:class="route.name.startsWith('discover-') ? 'text-black hover:underline' : 'text-link'"
			>
				Browse content
			</NuxtLink>
			<NuxtLink
				to="/hosting"
				class="px-2 py-2"
				:class="route.name.startsWith('hosting') ? 'text-black hover:underline' : 'text-link'"
			>
				Host a server
			</NuxtLink>
			<NuxtLink
				to="/app"
				class="px-2 py-2"
				:class="route.name.startsWith('app') ? 'text-black hover:underline' : 'text-link'"
			>
				Download application
			</NuxtLink>
			<div class="ml-auto flex items-center gap-1">
				<ButtonStyled type="transparent">
					<OverflowMenu
						v-if="auth.user && isStaff(auth.user)"
						class="flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
						position="bottom"
						direction="left"
						:dropdown-id="`${basePopoutId}-staff`"
						:aria-label="formatMessage(messages.createNew)"
						:options="[
							{
								id: 'review-projects',
								color: 'orange',
								link: '/moderation/',
							},
							{
								id: 'tech-review',
								color: 'orange',
								link: '/moderation/technical-review',
							},
							{
								id: 'review-reports',
								color: 'orange',
								link: '/moderation/reports',
							},
							{
								divider: true,
							},
							{
								id: 'file-lookup',
								link: '/admin/file_lookup',
							},
							{
								divider: true,
								shown: isAdmin(auth.user),
							},
							{
								id: 'user-lookup',
								color: 'primary',
								link: '/admin/user_email',
								shown: isAdmin(auth.user),
							},
							{
								id: 'affiliates',
								color: 'primary',
								link: '/admin/affiliates',
								shown: isAdmin(auth.user),
							},
							{
								id: 'servers-notices',
								color: 'primary',
								link: '/admin/servers/notices',
								shown: isAdmin(auth.user),
							},
							{
								id: 'servers-transfers',
								color: 'primary',
								link: '/admin/servers/transfers',
								shown: isAdmin(auth.user),
							},
							{
								id: 'servers-nodes',
								color: 'primary',
								action: (event) => $refs.modal_batch_credit.show(event),
								shown: isAdmin(auth.user),
							},
						]"
					>
						<ModrinthIcon aria-hidden="true" />
						▼
						<template #review-projects>
							<ScaleIcon aria-hidden="true" /> {{ formatMessage(messages.reviewProjects) }}
						</template>
						<template #tech-review>
							<ShieldAlertIcon aria-hidden="true" /> {{ formatMessage(messages.techReview) }}
						</template>
						<template #review-reports>
							<ReportIcon aria-hidden="true" /> {{ formatMessage(messages.reports) }}
						</template>
						<template #user-lookup>
							<UserSearchIcon aria-hidden="true" /> {{ formatMessage(messages.lookupByEmail) }}
						</template>
						<template #file-lookup>
							<FileIcon aria-hidden="true" /> {{ formatMessage(messages.fileLookup) }}
						</template>
						<template #servers-notices>
							<IssuesIcon aria-hidden="true" /> {{ formatMessage(messages.manageServerNotices) }}
						</template>
						<template #servers-transfers>
							<TransferIcon aria-hidden="true" /> Server transfers
						</template>
						<template #affiliates>
							<AffiliateIcon aria-hidden="true" /> {{ formatMessage(messages.manageAffiliates) }}
						</template>
						<template #servers-nodes>
							<ServerIcon aria-hidden="true" /> Credit server nodes
						</template>
					</OverflowMenu>
				</ButtonStyled>
				<ButtonStyled type="transparent">
					<OverflowMenu
						v-if="auth.user"
						class="flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
						position="bottom"
						direction="left"
						:dropdown-id="`${basePopoutId}-create`"
						:aria-label="formatMessage(messages.createNew)"
						:options="[
							{
								id: 'new-project',
								action: (event) => openProjectCreateModal(event),
							},
							{
								id: 'new-server-project',
								action: (event) => openProjectCreateModal(event, { type: 'server' }),
							},
							{
								id: 'new-collection',
								action: (event) => $refs.modal_collection_creation.show(event),
							},
							{ divider: true },
							{
								id: 'new-organization',
								action: (event) => $refs.modal_organization_creation.show(event),
							},
						]"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.publish) }}
						<template #new-project>
							<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.newProject) }}
						</template>
						<template #new-server-project>
							<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.newServerProject) }}
						</template>
						<!-- <template #import-project> <BoxImportIcon /> Import project </template>-->
						<template #new-collection>
							<LibraryIcon aria-hidden="true" /> {{ formatMessage(messages.newCollection) }}
						</template>
						<template #new-organization>
							<OrganizationIcon aria-hidden="true" /> {{ formatMessage(messages.newOrganization) }}
						</template>
					</OverflowMenu>
				</ButtonStyled>
				<OverflowMenu
					v-if="auth.user"
					:dropdown-id="`${basePopoutId}-user`"
					class="flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
					:options="userMenuOptions"
				>
					<Avatar :src="auth.user.avatar_url" aria-hidden="true" circle />
					<DropdownIcon class="h-5 w-5 text-secondary" />
					<template #profile>
						<UserIcon aria-hidden="true" /> {{ formatMessage(messages.profile) }}
					</template>
					<template #notifications>
						<BellIcon aria-hidden="true" /> {{ formatMessage(commonMessages.notificationsLabel) }}
					</template>
					<template #reports>
						<ReportIcon aria-hidden="true" /> {{ formatMessage(messages.activeReports) }}
					</template>
					<template #saved>
						<LibraryIcon aria-hidden="true" /> {{ formatMessage(commonMessages.collectionsLabel) }}
					</template>
					<template #servers>
						<ServerIcon aria-hidden="true" /> {{ formatMessage(messages.myServers) }}
					</template>
					<template #plus>
						<ArrowBigUpDashIcon aria-hidden="true" />
						{{ formatMessage(messages.upgradeToModrinthPlus) }}
					</template>
					<template #settings>
						<SettingsIcon aria-hidden="true" /> {{ formatMessage(commonMessages.settingsLabel) }}
					</template>
					<template #flags>
						<ToggleRightIcon aria-hidden="true" />
						{{ formatMessage(commonSettingsMessages.featureFlags) }}
					</template>
					<template #projects>
						<BoxIcon aria-hidden="true" /> {{ formatMessage(messages.projects) }}
					</template>
					<template #organizations>
						<OrganizationIcon aria-hidden="true" /> {{ formatMessage(messages.organizations) }}
					</template>
					<template #affiliate-links>
						<svg></svg>
						{{ formatMessage(commonMessages.affiliateLinksButton) }}
					</template>
					<template #revenue>
						<svg></svg>
						{{ formatMessage(messages.revenue) }}
					</template>
					<template #analytics>
						<svg></svg>
						{{ formatMessage(commonMessages.analyticsButton) }}
					</template>
					<template #moderation>
						<svg></svg>
						{{ formatMessage(commonMessages.moderationLabel) }}
					</template>
					<template #sign-out>
						<LogOutIcon aria-hidden="true" /> {{ formatMessage(commonMessages.signOutButton) }}
					</template>
				</OverflowMenu>
				<template v-else>
					<ButtonStyled color="brand">
						<nuxt-link to="/auth/sign-in">
							<LogInIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.signInButton) }}
						</nuxt-link>
					</ButtonStyled>
				</template>
			</div>
		</header>
		<main class="min-h-[calc(100vh-4.5rem-310.59px)]">
			<ProjectCreateModal v-if="auth.user" ref="modal_creation" />
			<CollectionCreateModal ref="modal_collection_creation" />
			<OrganizationCreateModal ref="modal_organization_creation" />
			<BatchCreditModal v-if="auth.user && isAdmin(auth.user)" ref="modal_batch_credit" />
			<slot id="main" />
		</main>
		<ModrinthFooter />
	</div>
</template>
<script setup>
import {
	AffiliateIcon,
	ArrowBigUpDashIcon,
	BellIcon,
	BoxIcon,
	DropdownIcon,
	FileIcon,
	IssuesIcon,
	LibraryIcon,
	LogInIcon,
	LogOutIcon,
	ModrinthIcon,
	OrganizationIcon,
	PlusIcon,
	ReportIcon,
	ScaleIcon,
	ServerIcon,
	SettingsIcon,
	ShieldAlertIcon,
	ToggleRightIcon,
	TransferIcon,
	UserIcon,
	UserSearchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	commonSettingsMessages,
	defineMessages,
	injectModrinthClient,
	OverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, UserBadge } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { provide, useTemplateRef } from 'vue'

import { getTaxThreshold } from '@/providers/creator-withdraw.ts'
import BatchCreditModal from '~/components/ui/admin/BatchCreditModal.vue'
import GeneratedStateErrorsBanner from '~/components/ui/banner/GeneratedStateErrorsBanner.vue'
import PreviewBanner from '~/components/ui/banner/PreviewBanner.vue'
import RussiaBanner from '~/components/ui/banner/RussiaBanner.vue'
import StagingBanner from '~/components/ui/banner/StagingBanner.vue'
import SubscriptionPaymentFailedBanner from '~/components/ui/banner/SubscriptionPaymentFailedBanner.vue'
import TaxComplianceBanner from '~/components/ui/banner/TaxComplianceBanner.vue'
import TaxIdMismatchBanner from '~/components/ui/banner/TaxIdMismatchBanner.vue'
import VerifyEmailBanner from '~/components/ui/banner/VerifyEmailBanner.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import OrganizationCreateModal from '~/components/ui/create/OrganizationCreateModal.vue'
import ProjectCreateModal from '~/components/ui/create/ProjectCreateModal.vue'
import ModrinthFooter from '~/components/ui/ModrinthFooter.vue'
import { errors as generatedStateErrors } from '~/generated/state.json'

const generatedState = useGeneratedState()

const country = useUserCountry()

const { formatMessage } = useVIntl()

const auth = await useAuth()
const user = await useUser()

const cosmetics = useCosmetics()
const flags = useFeatureFlags()

const config = useRuntimeConfig()
const route = useNativeRoute()
const router = useNativeRouter()
const link = config.public.siteUrl + route.path.replace(/\/+$/, '')
const client = injectModrinthClient()

const { data: payoutBalance } = useQuery({
	queryKey: ['payout', 'balance'],
	queryFn: () => client.labrinth.payout_v3.getBalance(),
	enabled: computed(() => !!auth.value.user),
})

const showTaxComplianceBanner = computed(() => {
	if (flags.value.testTaxForm && auth.value.user) return true
	const bal = payoutBalance.value
	if (!bal) return false
	const threshold = getTaxThreshold(generatedState.value?.taxComplianceThresholds)
	const thresholdMet = (bal.withdrawn_ytd ?? 0) >= threshold
	const status = bal.form_completion_status ?? 'unknown'
	const isComplete = status === 'complete'
	const isTinMismatch = status === 'tin-mismatch'
	return !!auth.value.user && thresholdMet && !isComplete && !isTinMismatch
})

const showTinMismatchBanner = computed(() => {
	const bal = payoutBalance.value
	if (!bal) return false
	const status = bal.form_completion_status ?? 'unknown'
	return !!auth.value.user && status === 'tin-mismatch'
})

const basePopoutId = useId()

const modalCreationRef = useTemplateRef('modal_creation')

function openProjectCreateModal(event, options) {
	modalCreationRef.value?.show(event, options)
}

provide('openProjectCreateModal', openProjectCreateModal)

const messages = defineMessages({
	toggleMenu: {
		id: 'layout.menu-toggle.action',
		defaultMessage: 'Toggle menu',
	},
	yourAvatarAlt: {
		id: 'layout.avatar.alt',
		defaultMessage: 'Your avatar',
	},
	modrinthHomePage: {
		id: 'layout.nav.modrinth-home-page',
		defaultMessage: 'Modrinth home page',
	},
	createNew: {
		id: 'layout.action.create-new',
		defaultMessage: 'Create new...',
	},
	publish: {
		id: 'layout.action.publish',
		defaultMessage: 'Publish',
	},
	reviewProjects: {
		id: 'layout.action.review-projects',
		defaultMessage: 'Project review',
	},
	techReview: {
		id: 'layout.action.tech-review',
		defaultMessage: 'Tech review',
	},
	reports: {
		id: 'layout.action.reports',
		defaultMessage: 'Review reports',
	},
	lookupByEmail: {
		id: 'layout.action.lookup-by-email',
		defaultMessage: 'Lookup by email',
	},
	fileLookup: {
		id: 'layout.action.file-lookup',
		defaultMessage: 'File lookup',
	},
	manageServerNotices: {
		id: 'layout.action.manage-server-notices',
		defaultMessage: 'Manage server notices',
	},
	manageAffiliates: {
		id: 'layout.action.manage-affiliates',
		defaultMessage: 'Manage affiliate links',
	},
	newProject: {
		id: 'layout.action.new-project',
		defaultMessage: 'New project',
	},
	newServerProject: {
		id: 'layout.action.new-server-project',
		defaultMessage: 'New server',
	},
	newCollection: {
		id: 'layout.action.new-collection',
		defaultMessage: 'New collection',
	},
	newOrganization: {
		id: 'layout.action.new-organization',
		defaultMessage: 'New organization',
	},
	profile: {
		id: 'layout.nav.profile',
		defaultMessage: 'Profile',
	},
	savedProjects: {
		id: 'layout.nav.saved-projects',
		defaultMessage: 'Saved projects',
	},
	upgradeToModrinthPlus: {
		id: 'layout.nav.upgrade-to-modrinth-plus',
		defaultMessage: 'Upgrade to Modrinth+',
	},
	projects: {
		id: 'layout.nav.projects',
		defaultMessage: 'Projects',
	},
	organizations: {
		id: 'layout.nav.organizations',
		defaultMessage: 'Organizations',
	},
	revenue: {
		id: 'layout.nav.revenue',
		defaultMessage: 'Revenue',
	},
	activeReports: {
		id: 'layout.nav.active-reports',
		defaultMessage: 'Active reports',
	},
	myServers: {
		id: 'layout.nav.my-servers',
		defaultMessage: 'My servers',
	},
	openMenu: {
		id: 'layout.mobile.open-menu',
		defaultMessage: 'Open menu',
	},
	closeMenu: {
		id: 'layout.mobile.close-menu',
		defaultMessage: 'Close menu',
	},
})

useHead({
	link: [
		{
			rel: 'canonical',
			href: link,
		},
	],
})
useSeoMeta({
	title: 'Modrinth',
	description: () =>
		formatMessage({
			id: 'layout.meta.description',
			defaultMessage:
				'Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. ' +
				'Discover and publish projects on Modrinth with a modern, easy to use interface and API.',
		}),
	publisher: 'Modrinth',
	themeColor: '#1bd96a',
	colorScheme: 'light',

	// OpenGraph
	ogTitle: 'Modrinth',
	ogSiteName: 'Modrinth',
	ogDescription: () =>
		formatMessage({
			id: 'layout.meta.og-description',
			defaultMessage: 'Discover and publish Minecraft content!',
		}),
	ogType: 'website',
	ogImage: 'https://cdn.modrinth.com/modrinth-new.png',
	ogUrl: link,

	// Twitter
	twitterCard: 'summary',
	twitterSite: '@modrinth',
})

const isMobileMenuOpen = ref(false)
const isBrowseMenuOpen = ref(false)

const userMenuOptions = computed(() => {
	const user = auth.value.user
	if (!user) return []

	let options = [
		{
			id: 'profile',
			link: `/user/${user.username}`,
		},
		{
			id: 'plus',
			link: '/plus',
			color: 'purple',
			shown: !flags.value.hidePlusPromoInUserMenu && !isPermission(user.badges, 1 << 0),
		},
		{
			id: 'servers',
			link: '/hosting/manage',
		},
		{
			id: 'flags',
			link: '/settings/flags',
			shown: flags.value.developerMode,
		},
		{
			id: 'settings',
			link: '/settings',
		},
	]

	// TODO: Only show if user has projects
	options = [
		...options,
		{
			divider: true,
		},
		{
			id: 'notifications',
			link: '/dashboard/notifications',
		},
		{
			id: 'reports',
			link: '/dashboard/reports',
		},
		{
			id: 'saved',
			link: '/dashboard/collections',
		},
		{
			divider: true,
		},
		{
			id: 'projects',
			link: '/dashboard/projects',
		},
		{
			id: 'organizations',
			link: '/dashboard/organizations',
		},
		{
			id: 'analytics',
			link: '/dashboard/analytics',
		},
		{
			id: 'affiliate-links',
			link: '/dashboard/affiliate-links',
			shown: user.badges & UserBadge.AFFILIATE,
		},
		{
			id: 'revenue',
			link: '/dashboard/revenue',
		},
	]

	options = [
		...options,
		{
			divider: true,
		},
		{
			id: 'sign-out',
			color: 'danger',
			action: () => logoutUser(),
			hoverFilled: true,
		},
	]
	return options
})

const isRussia = computed(() => country.value === 'ru')

const rCount = ref(0)

const randomProjects = ref([])
const disableRandomProjects = ref(false)

const disableRandomProjectsForRoute = computed(
	() =>
		route.name.startsWith('hosting') ||
		route.name.includes('settings') ||
		route.name.includes('admin'),
)

async function onKeyDown(event) {
	if (disableRandomProjects.value || disableRandomProjectsForRoute.value) {
		return
	}

	if (event.key === 'r') {
		rCount.value++

		if (randomProjects.value.length < 3) {
			randomProjects.value = await client.labrinth.projects_v2.getRandom(50).catch((err) => {
				console.error(err)
				return []
			})
		}
	}

	if (rCount.value >= 40) {
		rCount.value = 0
		const randomProject = randomProjects.value[0]
		await router.push(`/project/${randomProject.slug}`)
		randomProjects.value.splice(0, 1)
	}
}

function onKeyUp(event) {
	if (event.key === 'r') {
		rCount.value = 0
	}
}

onMounted(() => {
	if (window && import.meta.client) {
		window.history.scrollRestoration = 'auto'
	}

	runAnalytics()

	window.addEventListener('keydown', onKeyDown)
	window.addEventListener('keyup', onKeyUp)
})

watch(
	() => route.path,
	() => {
		isMobileMenuOpen.value = false
		isBrowseMenuOpen.value = false

		if (import.meta.client) {
			document.body.style.overflowY = 'scroll'
			document.body.setAttribute('tabindex', '-1')
			document.body.removeAttribute('tabindex')
		}

		runAnalytics()
	},
)

async function logoutUser() {
	await logout()
}

function runAnalytics() {
	const config = useRuntimeConfig()
	const replacedUrl = config.public.apiBaseUrl.replace('v2/', '')

	try {
		setTimeout(() => {
			$fetch(`${replacedUrl}analytics/view`, {
				method: 'POST',
				body: {
					url: window.location.href,
				},
				headers: {
					Authorization: auth.value.token,
				},
			})
				.then(() => {})
				.catch(() => {})
		})
	} catch (e) {
		console.error(`Sending analytics failed (CORS error? If so, ignore)`, e)
	}
}
</script>

<style lang="scss">
@import '~/assets/styles/global.scss';
// @import '@modrinth/assets';

body {
	background: #c4e8c9;
	overflow: auto;
}

.header-background {
	background-color: var(--color-brand);
	background-image: linear-gradient(
		to bottom,
		var(--color-green-300),
		var(--color-green-500),
		var(--color-green-600)
	);
	border-radius: 8px 8px 0 0;
	border: 1px solid var(--color-green-500);
	margin-top: 1rem;
}

.selected-header-item {
	background: linear-gradient(to bottom, rgba(0, 0, 0, 0.1), rgba(0, 0, 0, 0.15));
}

.layout {
	background: white;
	background-image: linear-gradient(to bottom, white, #ececec);
	max-width: 1200px;
	width: 1200px;
	min-wdith: 1200px;
	margin-inline: auto;
	min-height: 100vh;
	display: block;
	border-radius: 8px 8px 0 0;

	@media screen and (min-width: 1024px) {
		min-height: calc(100vh - var(--spacing-card-bg));
	}

	main {
		grid-area: main;
	}
}

@media (min-width: 1024px) {
	.layout {
		main {
			.alpha-alert {
				margin: 1rem;

				.wrapper {
					padding: 1rem 2rem 1rem 1rem;
				}
			}
		}
	}
}

@media (max-width: 1200px) {
	.app-btn {
		display: none;
	}
}

.over-the-top-random-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 0;
	animation:
		tilt-shaking calc(0.2s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite,
		translate-x-shaking calc(0.3s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite,
		translate-y-shaking calc(0.25s / (max((var(--_r-count) - 20), 1) / 20)) linear infinite;

	&.threshold {
		opacity: 1;
	}

	&.rings-expand {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}

		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}

		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: calc(1 + max((var(--_r-count) - 20), 0) * 0.1);
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}

@keyframes tilt-shaking {
	0% {
		rotate: 0deg;
	}

	25% {
		rotate: calc(1deg * (var(--_r-count) - 20));
	}

	50% {
		rotate: 0deg;
	}

	75% {
		rotate: calc(-1deg * (var(--_r-count) - 20));
	}

	100% {
		rotate: 0deg;
	}
}

@keyframes translate-x-shaking {
	0% {
		translate: 0;
	}

	25% {
		translate: calc(2px * (var(--_r-count) - 20));
	}

	50% {
		translate: 0;
	}

	75% {
		translate: calc(-2px * (var(--_r-count) - 20));
	}

	100% {
		translate: 0;
	}
}

@keyframes translate-y-shaking {
	0% {
		transform: translateY(0);
	}

	25% {
		transform: translateY(calc(2px * (var(--_r-count) - 20)));
	}

	50% {
		transform: translateY(0);
	}

	75% {
		transform: translateY(calc(-2px * (var(--_r-count) - 20)));
	}

	100% {
		transform: translateY(0);
	}
}
</style>
