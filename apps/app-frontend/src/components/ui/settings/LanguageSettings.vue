<script setup lang="ts">
import { LanguageSelector, LOCALES } from '@modrinth/ui'
import { ref, watch } from 'vue'

import i18n from '@/i18n.config'
import { get, set } from '@/helpers/settings.ts'

const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

const $isChanging = ref(false)

async function onLocaleChange(newLocale: string) {
	if (settings.value.locale === newLocale) return

	$isChanging.value = true
	try {
		i18n.global.locale.value = newLocale
		settings.value.locale = newLocale
	} finally {
		$isChanging.value = false
	}
}
</script>

<template>
	<h2 class="m-0 text-lg font-extrabold text-contrast">Language</h2>
	<p class="m-0 mt-1 mb-4">
		Select your preferred language for the Modrinth App. Translations are contributed by
		volunteers on
		<a href="https://translate.modrinth.com" class="text-link hover:underline">Crowdin</a>.
	</p>

	<LanguageSelector
		:current-locale="settings.locale"
		:locales="LOCALES"
		:on-locale-change="onLocaleChange"
		:is-changing="$isChanging"
	/>
</template>
