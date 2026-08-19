<template>
	<div>
		<section class="universal-card">
			<LanguageSettings ref="languageSettings" product="website" />
		</section>
		<UnsavedChangesPopup
			:original="languageSettings?.originalState ?? emptyLanguageState"
			:modified="languageSettings?.modifiedState ?? emptyLanguageState"
			:saving="languageSettings?.saving ?? false"
			@reset="languageSettings?.reset()"
			@save="languageSettings?.save()"
		/>
	</div>
</template>

<script setup lang="ts">
import {
	commonSettingsMessages,
	LanguageSettings,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const languageSettings = ref<InstanceType<typeof LanguageSettings> | null>(null)
const emptyLanguageState = { locale: '' }

useHead({
	title: () => `${formatMessage(commonSettingsMessages.language)} - Modrinth`,
})
</script>
