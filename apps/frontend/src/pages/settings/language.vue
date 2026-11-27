<template>
	<section class="universal-card">
		<LanguageSettings product="website" />
	</section>
</template>

<script setup lang="ts">
import { commonSettingsMessages, LanguageSettings, useVIntl } from '@modrinth/ui'

const { formatMessage } = useVIntl()

useHead({
	title: () => `${formatMessage(commonSettingsMessages.language)} - Modrinth`,
})
</script>
