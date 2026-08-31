<template>
	<NuxtLayout>
		<NuxtRouteAnnouncer />
		<ClientOnly>
			<LoadingBar />
		</ClientOnly>
		<NotificationPanel />
		<AccountSwitchOverlay :show="isSwitchingAccount" />
		<AdsConsentNotification />
		<I18nDebugPanel />
		<NuxtPage />
		<div id="teleports"></div>
	</NuxtLayout>
</template>
<script setup lang="ts">
import {
	AccountSwitchOverlay,
	I18nDebugPanel,
	injectI18n,
	LoadingBar,
	NotificationPanel,
} from '@modrinth/ui'

import AdsConsentNotification from '~/components/ui/AdsConsentNotification.vue'
import { setupProviders } from '~/providers/setup.ts'

import {
	hydrateStoredAccounts,
	rememberStoredAccount,
	rememberStoredAccountAppearance,
	useIsSwitchingAccount,
} from './composables/accounts'
import { useAuth } from './composables/auth'

const auth = await useAuth()
const { userPreferences } = setupProviders(auth)
const cosmetics = useCosmetics()
const theme = useTheme()
const { locale, setLocale } = injectI18n()
const isSwitchingAccount = useIsSwitchingAccount()

// initAuth doesn't run again after SSR, so stash the current account for switching
watch(
	auth,
	({ user, token }) => {
		if (user && token) {
			rememberStoredAccount(user, token)
		}
	},
	{ immediate: true },
)

onMounted(hydrateStoredAccounts)

watch(
	userPreferences.preferences,
	(preferences) => {
		if (!preferences) return

		if (theme.syncAcrossDevices) {
			theme.applyAccountAppearance(preferences.appearance)
		}

		const userId = auth.value.user?.id
		if (userId) {
			rememberStoredAccountAppearance(userId, preferences.appearance)
		}

		if (locale.value !== preferences.localization.locale) {
			void setLocale(preferences.localization.locale)
		}

		cosmetics.value.searchDisplayMode.mod = preferences.layouts.mods === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.plugin =
			preferences.layouts.plugins === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.datapack =
			preferences.layouts.datapacks === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.shader =
			preferences.layouts.shaders === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.resourcepack =
			preferences.layouts.resourcepacks === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.modpack =
			preferences.layouts.modpacks === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.server =
			preferences.layouts.servers === 'rows' ? 'list' : 'grid'
		cosmetics.value.searchDisplayMode.user = preferences.layouts.users === 'rows' ? 'list' : 'grid'
		cosmetics.value.rightSearchLayout = preferences.sidebars.right_aligned_search
		cosmetics.value.leftContentLayout = preferences.sidebars.left_aligned_content
	},
	{ immediate: true },
)
</script>
