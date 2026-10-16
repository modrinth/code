<template>
	<SharedLanguageSettings ref="languageSettings" product="app" :persist-locale="persistLocale" />
</template>

<script setup lang="ts">
import { LanguageSettings as SharedLanguageSettings } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, ref } from 'vue'

import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { get, set } from '@/helpers/settings.ts'

const settingsModal = injectAppSettingsModal()
const languageSettings = ref<InstanceType<typeof SharedLanguageSettings> | null>(null)

onMounted(() => {
	settingsModal.registerUnsavedChangesController({
		hasChanges: () => languageSettings.value?.hasChanges ?? false,
		getOriginal: () => languageSettings.value?.originalState ?? {},
		getModified: () => languageSettings.value?.modifiedState ?? {},
		isSaving: () => languageSettings.value?.saving ?? false,
		reset: () => languageSettings.value?.reset(),
		save: () => languageSettings.value?.save(),
	})
})

onBeforeUnmount(() => {
	settingsModal.registerUnsavedChangesController(null)
})

async function persistLocale(locale: string): Promise<void> {
	const settings = await get()
	if (settings.locale === locale) return
	await set({ ...settings, locale })
}
</script>
