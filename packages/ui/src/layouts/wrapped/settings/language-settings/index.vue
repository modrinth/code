<template>
	<div>
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(commonSettingsMessages.language) }}
		</h2>

		<Admonition type="warning" class="mb-4 mt-2">
			{{ formatMessage(languageSelectorMessages.languageWarning, { platform }) }}
		</Admonition>

		<p class="m-0 mb-4 text-secondary">
			<IntlFormatted
				:message-id="languageSelectorMessages.languagesDescription"
				:values="{ platform }"
			>
				<template #~crowdin-link="{ children }">
					<AutoLink to="https://translate.modrinth.com" class="text-link">
						<component :is="() => children" />
					</AutoLink>
				</template>
			</IntlFormatted>
		</p>

		<LanguageSettingsSelector
			:product="product"
			:current-locale="locale"
			:locales="LOCALES"
			:on-locale-change="onLocaleChange"
			:coverage-by-locale="languageCoverage[product]"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { Admonition, AutoLink, IntlFormatted } from '#ui/components/base'
import { LOCALES, useVIntl } from '#ui/composables'
import { injectI18n, injectUserPreferences } from '#ui/providers'
import { commonSettingsMessages, languageSelectorMessages } from '#ui/utils'

import { languageCoverage } from './language-settings-coverage.generated'
import LanguageSettingsSelector from './language-settings-selector.vue'

const props = defineProps<{
	product: 'app' | 'website'
	persistLocale?: (locale: string) => void | Promise<void>
}>()

const { formatMessage } = useVIntl()
const { locale, setLocale } = injectI18n()
const { updatePreferences } = injectUserPreferences()
const platform = computed(() =>
	formatMessage(
		props.product === 'app'
			? languageSelectorMessages.platformApp
			: languageSelectorMessages.platformSite,
	),
)

async function onLocaleChange(newLocale: string): Promise<void> {
	await updatePreferences({ localization: { locale: newLocale } })
	await setLocale(newLocale, { persist: true })
	await props.persistLocale?.(newLocale)
}
</script>
