<script setup lang="ts">
import {
	Admonition,
	commonSettingsMessages,
	IntlFormatted,
	LanguageSelector,
	languageSelectorMessages,
	LOCALES,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const { locale, setLocale } = useI18n()

const platform = formatMessage(languageSelectorMessages.platformSite)

const $isChanging = ref(false)

async function onLocaleChange(newLocale: string) {
	if (locale.value === newLocale) return

	$isChanging.value = true
	try {
		await setLocale(newLocale)
	} finally {
		$isChanging.value = false
	}
}
</script>

<template>
	<div>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(commonSettingsMessages.language) }}</h2>

			<Admonition type="warning">
				{{ formatMessage(languageSelectorMessages.languageWarning, { platform }) }}
			</Admonition>

			<div class="card-description mt-4">
				<IntlFormatted
					:message-id="languageSelectorMessages.languagesDescription"
					:values="{ platform }"
				>
					<template #~crowdin-link="{ children }">
						<a href="https://translate.modrinth.com">
							<component :is="() => children" />
						</a>
					</template>
				</IntlFormatted>
			</div>

			<LanguageSelector
				:current-locale="locale"
				:locales="LOCALES"
				:on-locale-change="onLocaleChange"
				:is-changing="$isChanging"
			/>
		</section>
	</div>
</template>

<style scoped lang="scss">
.card-description {
	margin-bottom: calc(var(--spacing-card-sm) + var(--spacing-card-md));

	a {
		color: var(--color-link);

		&:hover {
			color: var(--color-link-hover);
		}

		&:active {
			color: var(--color-link-active);
		}
	}
}
</style>
