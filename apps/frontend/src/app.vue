<template>
	<NuxtLayout>
		<NuxtRouteAnnouncer />
		<ClientOnly>
			<LoadingBar />
		</ClientOnly>
		<NotificationPanel />
		<AdsConsentNotification />
		<I18nDebugPanel />
		<NuxtPage />
	</NuxtLayout>
</template>
<script setup lang="ts">
import { I18nDebugPanel, injectI18n, LoadingBar, NotificationPanel } from '@modrinth/ui'

import AdsConsentNotification from '~/components/ui/AdsConsentNotification.vue'
import { isDarkTheme } from '~/plugins/theme/index.ts'
import { setupProviders } from '~/providers/setup.ts'

import { useAuth } from './composables/auth'

const auth = await useAuth()
const userPreferences = setupProviders(auth)
const cosmetics = useCosmetics()
const theme = useTheme()
const { locale, setLocale } = injectI18n()

watch(
	userPreferences.preferences,
	(preferences) => {
		if (!preferences) return

		if (theme.syncAcrossDevices) {
			if (isDarkTheme(preferences.appearance.theme)) {
				theme.preferences.dark = preferences.appearance.theme
			} else {
				theme.preferences.light = preferences.appearance.theme
			}
			theme.preferred = preferences.appearance.auto ? 'system' : preferences.appearance.theme
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
