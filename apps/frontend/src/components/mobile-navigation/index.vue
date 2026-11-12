<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BellIcon,
	DashboardIcon,
	HamburgerIcon,
	HomeIcon,
	LibraryIcon,
	LogInIcon,
	LogOutIcon,
	MoonIcon,
	PlusIcon,
	ScaleIcon,
	SearchIcon,
	ServerIcon,
	ServerStackIcon,
	SettingsIcon,
	SunIcon,
	ToggleRightIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonLink,
	commonMessages,
	commonSettingsMessages,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'

import { getProjectTypeMessage } from '~/utils/i18n-project-type.ts'

import MobileNavigationSheet from './sheet.vue'
import MobileNavigationTab from './tab.vue'

defineProps<{
	user: Labrinth.Users.v2.User | null
}>()

const emit = defineEmits<{
	signIn: []
	signOut: []
	createProject: []
	changeTheme: []
}>()

const isBrowseMenuOpen = defineModel<boolean>('browseMenuOpen', { default: false })
const isAccountMenuOpen = ref(false)
const isAnyMenuOpen = computed(() => isBrowseMenuOpen.value || isAccountMenuOpen.value)

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()
const theme = useTheme()
const route = useNativeRoute()

const discoverRoutes = computed(() =>
	(['mod', 'resourcepack', 'datapack', 'shader', 'modpack', 'plugin', 'server'] as const).map(
		(type) => ({
			label: formatMessage(getProjectTypeMessage(type, true)),
			href: `/discover/${type}s`,
		}),
	),
)

watch(
	() => route.path,
	() => closeMenus(),
)

function isTabActive(path: string) {
	return !isAnyMenuOpen.value && route.path === path
}

function toggleBrowseMenu() {
	isBrowseMenuOpen.value = !isBrowseMenuOpen.value
	if (isBrowseMenuOpen.value) isAccountMenuOpen.value = false
}

function toggleAccountMenu() {
	isAccountMenuOpen.value = !isAccountMenuOpen.value
	if (isAccountMenuOpen.value) isBrowseMenuOpen.value = false
}

function closeMenus() {
	isBrowseMenuOpen.value = false
	isAccountMenuOpen.value = false
}

const messages = defineMessages({
	home: {
		id: 'layout.nav.home',
		defaultMessage: 'Home',
	},
	search: {
		id: 'layout.nav.search',
		defaultMessage: 'Search',
	},
	hostAServer: {
		id: 'layout.nav.host-a-server',
		defaultMessage: 'Host a server',
	},
	openMenu: {
		id: 'layout.mobile.open-menu',
		defaultMessage: 'Open menu',
	},
	closeMenu: {
		id: 'layout.mobile.close-menu',
		defaultMessage: 'Close menu',
	},
	yourAvatarAlt: {
		id: 'layout.avatar.alt',
		defaultMessage: 'Your avatar',
	},
	changeTheme: {
		id: 'layout.action.change-theme',
		defaultMessage: 'Change theme',
	},
})
</script>

<template>
	<header class="hidden [@media(pointer:coarse)_and_(max-width:640px)]:flex">
		<MobileNavigationSheet
			:open="isBrowseMenuOpen"
			@focusin="isBrowseMenuOpen = true"
			@focusout="isBrowseMenuOpen = false"
		>
			<div class="mx-auto grid max-w-[532px] gap-3 p-4 min-[354px]:grid-cols-2">
				<ButtonLink
					v-for="discoverRoute in discoverRoutes"
					:key="discoverRoute.href"
					:to="discoverRoute.href"
					size="lg"
				>
					{{ discoverRoute.label }}
				</ButtonLink>
			</div>
		</MobileNavigationSheet>
		<MobileNavigationSheet
			:open="isAccountMenuOpen"
			@focusin="isAccountMenuOpen = true"
			@focusout="isAccountMenuOpen = false"
		>
			<div
				class="mx-auto grid max-w-[532px] p-4 pb-0 transition-opacity duration-150 ease-in-out"
				:class="
					isAccountMenuOpen ? 'visible opacity-100' : 'pointer-events-none invisible opacity-0'
				"
			>
				<ButtonLink
					v-if="user"
					:to="`/user/${user.username}`"
					size="lg"
					class="!h-auto !gap-2 !py-2"
				>
					<Avatar
						:src="user.avatar_url"
						size="2.25rem"
						:alt="formatMessage(messages.yourAvatarAlt)"
						aria-hidden="true"
						circle
					/>
					<div>
						<div>@{{ user.username }}</div>
						<div>{{ formatMessage(commonMessages.visitYourProfile) }}</div>
					</div>
				</ButtonLink>
				<Button v-else type="colored" color="brand" size="lg" @click="emit('signIn')">
					<LogInIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.signInButton) }}
				</Button>
			</div>
			<div class="mx-auto grid max-w-[532px] gap-3 p-4">
				<template v-if="user">
					<Button type="colored" color="red" size="lg" @click="emit('signOut')">
						<LogOutIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.signOutButton) }}
					</Button>
					<Button size="lg" @click="emit('createProject')">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.createAProjectButton) }}
					</Button>
					<ButtonLink to="/dashboard/collections" size="lg">
						<LibraryIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.collectionsLabel) }}
					</ButtonLink>
					<ButtonLink to="/hosting/manage" size="lg">
						<ServerIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.serversLabel) }}
					</ButtonLink>
					<ButtonLink
						v-if="user.role === 'moderator' || user.role === 'admin'"
						to="/moderation"
						size="lg"
					>
						<ScaleIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.moderationLabel) }}
					</ButtonLink>
					<ButtonLink v-if="flags.developerMode" to="/settings/flags" size="lg">
						<ToggleRightIcon aria-hidden="true" />
						{{ formatMessage(commonSettingsMessages.featureFlags) }}
					</ButtonLink>
				</template>
				<ButtonLink to="/settings" size="lg">
					<SettingsIcon aria-hidden="true" />
					{{ formatMessage(commonMessages.settingsLabel) }}
				</ButtonLink>
				<Button size="lg" @click="emit('changeTheme')">
					<MoonIcon v-if="theme.active === 'light'" aria-hidden="true" />
					<SunIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.changeTheme) }}
				</Button>
			</div>
		</MobileNavigationSheet>
		<nav
			class="fixed bottom-0 left-0 z-[11] box-border flex h-[calc(var(--size-mobile-navbar-height)+env(safe-area-inset-bottom))] w-full items-center justify-between border border-b-0 border-solid border-surface-5 bg-surface-3 px-2 pb-[env(safe-area-inset-bottom)] transition-[border-radius,box-shadow] duration-300 ease-out after:absolute after:-left-px after:-right-px after:bottom-0 after:h-[300px] after:translate-y-full after:bg-surface-3"
			:class="
				isAnyMenuOpen ? 'rounded-none' : 'rounded-t-2xl shadow-[0_0_20px_2px_rgba(0,0,0,0.2)]'
			"
		>
			<MobileNavigationTab to="/" :label="formatMessage(messages.home)" :active="isTabActive('/')">
				<HomeIcon aria-hidden="true" />
			</MobileNavigationTab>
			<MobileNavigationTab
				:label="formatMessage(messages.search)"
				:active="isBrowseMenuOpen"
				:icon-only="!!user"
				@click="toggleBrowseMenu()"
			>
				<SearchIcon aria-hidden="true" />
				<template v-if="!user">{{ formatMessage(messages.search) }}</template>
			</MobileNavigationTab>
			<MobileNavigationTab
				to="/hosting"
				:label="formatMessage(messages.hostAServer)"
				:active="isTabActive('/hosting')"
			>
				<ServerStackIcon aria-hidden="true" />
			</MobileNavigationTab>
			<template v-if="user">
				<MobileNavigationTab
					to="/dashboard/notifications"
					:label="formatMessage(commonMessages.notificationsLabel)"
					:active="isTabActive('/dashboard/notifications')"
					@click="closeMenus()"
				>
					<BellIcon aria-hidden="true" />
				</MobileNavigationTab>
				<MobileNavigationTab
					to="/dashboard"
					:label="formatMessage(commonMessages.dashboardLabel)"
					:active="isTabActive('/dashboard')"
					@click="closeMenus()"
				>
					<DashboardIcon aria-hidden="true" />
				</MobileNavigationTab>
			</template>
			<MobileNavigationTab
				:label="
					isAccountMenuOpen ? formatMessage(messages.closeMenu) : formatMessage(messages.openMenu)
				"
				:active="isAccountMenuOpen"
				@click="toggleAccountMenu()"
			>
				<template v-if="!user">
					<HamburgerIcon v-if="!isAccountMenuOpen" aria-hidden="true" />
					<XIcon v-else aria-hidden="true" />
				</template>
				<Avatar
					v-else
					:src="user.avatar_url"
					:alt="formatMessage(messages.yourAvatarAlt)"
					aria-hidden="true"
					circle
				/>
			</MobileNavigationTab>
		</nav>
	</header>
</template>
