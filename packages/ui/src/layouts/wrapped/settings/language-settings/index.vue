<template>
	<div>
		<h2 class="m-0 text-xl font-semibold text-contrast">
			{{ formatMessage(commonSettingsMessages.language) }}
		</h2>

		<Admonition type="warning" class="mb-4 mt-2">
			{{ formatMessage(languageSelectorMessages.languageWarning, { platform }) }}
		</Admonition>

		<Admonition
			v-if="locale !== SOURCE_LOCALE"
			type="info"
			class="mb-4"
			inline-actions
			:header="formatMessage(messages.feedbackTitle)"
			:body="formatMessage(messages.feedbackDescription, { platform })"
		>
			<template #actions>
				<Button type="colored" color="blue" @click="feedbackModal?.show()">
					<ReportIcon />
					{{ formatMessage(messages.feedbackButton) }}
				</Button>
			</template>
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
			:current-locale="current.locale"
			:locales="LOCALES"
			:on-locale-change="onLocaleChange"
			:is-changing="saving"
			:coverage-by-locale="languageCoverage[product]"
		/>

		<TranslationFeedbackModal ref="feedbackModal" :product="product" />
	</div>
</template>

<script setup lang="ts">
import { ReportIcon } from '@modrinth/assets'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { Admonition, AutoLink, IntlFormatted } from '#ui/components/base'
import { Button } from '#ui/components/base/buttons'
import { defineMessages, LOCALES, useVIntl } from '#ui/composables'
import { injectI18n, injectUserPreferences } from '#ui/providers'
import { commonSettingsMessages, languageSelectorMessages, useSavable } from '#ui/utils'

import { languageCoverage } from './language-settings-coverage.generated'
import LanguageSettingsSelector from './language-settings-selector.vue'
import TranslationFeedbackModal from './translation-feedback-modal.vue'

const SOURCE_LOCALE = 'en-US'

const props = defineProps<{
	product: 'app' | 'website'
	persistLocale?: (locale: string) => void | Promise<void>
}>()

const { formatMessage } = useVIntl()
const { locale, setLocale } = injectI18n()
const { preferences, updatePreferences } = injectUserPreferences()
const platform = computed(() =>
	formatMessage(
		props.product === 'app'
			? languageSelectorMessages.platformApp
			: languageSelectorMessages.platformSite,
	),
)
const messages = defineMessages({
	feedbackTitle: {
		id: 'settings.language.feedback.title',
		defaultMessage: 'Spotted a translation mistake?',
	},
	feedbackDescription: {
		id: 'settings.language.feedback.description',
		defaultMessage: 'Let our translators know about text in the {platform} that reads wrong.',
	},
	feedbackButton: {
		id: 'settings.language.feedback.button',
		defaultMessage: 'Report a translation issue',
	},
})

const feedbackModal = ref<InstanceType<typeof TranslationFeedbackModal> | null>(null)
const persistedLocale = ref(locale.value)
let localeChangeQueue = Promise.resolve()

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	() => ({ locale: persistedLocale.value }),
	async () => {
		await updatePreferences({ localization: { locale: current.value.locale } })
		await queueLocaleChange(current.value.locale, true)
		await props.persistLocale?.(current.value.locale)
		persistedLocale.value = current.value.locale
	},
)

function queueLocaleChange(newLocale: string, persist = false): Promise<void> {
	const request = localeChangeQueue.then(() => Promise.resolve(setLocale(newLocale, { persist })))
	localeChangeQueue = request.catch(() => undefined)
	return request
}

function onLocaleChange(newLocale: string): void {
	current.value.locale = newLocale
	void queueLocaleChange(newLocale).catch(() => undefined)
}

function resetLanguageSettings(): void {
	reset()
	void queueLocaleChange(current.value.locale).catch(() => undefined)
}

async function saveLanguageSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

watch(
	preferences,
	(value) => {
		if (!value || hasChanges.value) return
		persistedLocale.value = value.localization.locale
	},
	{ immediate: true, flush: 'sync' },
)

onBeforeUnmount(() => {
	if (hasChanges.value || locale.value !== persistedLocale.value) {
		void queueLocaleChange(persistedLocale.value).catch(() => undefined)
	}
})

defineExpose({
	originalState: saved,
	modifiedState: changes,
	hasChanges,
	saving,
	reset: resetLanguageSettings,
	save: saveLanguageSettings,
})
</script>
