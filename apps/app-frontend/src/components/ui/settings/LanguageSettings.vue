<script setup lang="ts">
import {
	Admonition,
	AutoLink,
	IntlFormatted,
	LanguageSelector,
	languageSelectorMessages,
	LOCALES,
	useVIntl,
} from '@modrinth/ui'
import { ref, watch } from 'vue'

import { get, set } from '@/helpers/settings.ts'
import i18n from '@/i18n.config'

const { formatMessage } = useVIntl()

const platform = formatMessage(languageSelectorMessages.platformApp)

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

	<Admonition type="warning" class="mt-2 mb-4">
		{{ formatMessage(languageSelectorMessages.languageWarning, { platform }) }}
	</Admonition>

	<p class="m-0 mb-4">
		<IntlFormatted
			:message-id="languageSelectorMessages.languagesDescription"
			:values="{ platform }"
		>
			<template #~crowdin-link="{ children }">
				<AutoLink to="https://translate.modrinth.com">
					<component :is="() => children" />
				</AutoLink>
			</template>
		</IntlFormatted>
	</p>

	<LanguageSelector
		:current-locale="settings.locale"
		:locales="LOCALES"
		:on-locale-change="onLocaleChange"
		:is-changing="$isChanging"
	/>
</template>
