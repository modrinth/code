<template>
	<SharedLanguageSettings product="app" :persist-locale="persistLocale" />
</template>

<script setup lang="ts">
import { LanguageSettings as SharedLanguageSettings } from '@modrinth/ui'

import { get, set } from '@/helpers/settings.ts'

async function persistLocale(locale: string): Promise<void> {
	const settings = await get()
	if (settings.locale === locale) return
	await set({ ...settings, locale })
}
</script>
