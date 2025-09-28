<script setup lang="ts">
import {
	IssuesIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import { Admonition, Button, commonSettingsMessages } from '@modrinth/ui'
import { IntlFormatted } from '@vintl/vintl/components'
import Fuse from 'fuse.js'

import { isModifierKeyDown } from '~/helpers/events.ts'

const vintl = useVIntl()
const { formatMessage } = vintl

const messages = defineMessages({
	languagesDescription: {
		id: 'settings.language.description',
		defaultMessage:
			'Choose your preferred language for the site. Translations are contributed by volunteers <crowdin-link>on Crowdin</crowdin-link>.',
	},
	automaticLocale: {
		id: 'settings.language.languages.automatic',
		defaultMessage: 'Sync with the system language',
	},
	loadFailed: {
		id: 'settings.language.languages.load-failed',
		defaultMessage: 'Cannot load this language. Try again in a bit.',
	},
	languageLabelApplying: {
		id: 'settings.language.languages.language-label-applying',
		defaultMessage: '{label}. Applying...',
	},
	languageLabelError: {
		id: 'settings.language.languages.language-label-error',
		defaultMessage: '{label}. Error',
	},
	languageWarning: {
		id: 'settings.language.warning',
		defaultMessage:
			'Changing the site language may cause some content to appear in English if a translation is not available. The site is not yet fully translated, so some content may remain in English for certain languages. We are still working on improving our localization system, so occasionally content may appear broken.',
	},
	experimentalBadge: {
		id: 'settings.language.experimental-badge',
		defaultMessage: 'Experimental',
	},
	searchPlaceholder: {
		id: 'settings.language.search.placeholder',
		defaultMessage: 'Search languages...',
	},
})

type AutomaticLocale = {
	auto: true
	tag: string
}

type CommonLocale = {
	auto?: never
	tag: string
	displayName: string
	translatedName: string
	isRTL?: boolean
	completionPercentage?: number
}

type Locale = AutomaticLocale | CommonLocale

const $defaultNames = useDisplayNames(() => vintl.defaultLocale)
const $translatedNames = useDisplayNames(() => vintl.locale)

const $locales = computed(() => {
	const locales: Locale[] = []

	locales.push({
		auto: true,
		tag: 'auto',
	})

	const currentLocaleData = vintl.availableLocales.find((l) => l.tag === vintl.locale)
	const currentLanguages = currentLocaleData?.meta?.languages as Record<string, string> | undefined

	const enUSLocale = vintl.availableLocales.find((l) => l.tag === vintl.defaultLocale)
	const referenceKeyCount = enUSLocale?.meta?.keys as number | undefined

	for (const locale of vintl.availableLocales) {
		let displayName = locale.meta?.displayName
		if (displayName == null) {
			displayName = $defaultNames.value.of(locale.tag) ?? locale.tag
		}

		let translatedName = currentLanguages?.[locale.tag] as string | null
		if (translatedName == null) {
			translatedName = $translatedNames.value.of(locale.tag) ?? locale.tag
		}

		let completionPercentage: number | undefined
		if (referenceKeyCount && locale.meta?.keys) {
			completionPercentage = Math.round((locale.meta.keys / referenceKeyCount) * 100)
		}

		// Skip languages with undefined or 0 completion percentage
		if (completionPercentage === undefined || completionPercentage === 0) {
			continue
		}

		locales.push({
			tag: locale.tag,
			displayName,
			translatedName,
			isRTL: locale.meta?.isRTL === true,
			completionPercentage,
		})
	}

	// Sort locales by completion percentage (descending), with auto locale first
	return locales.sort((a, b) => {
		// Auto locale always comes first
		if (a.auto) return -1
		if (b.auto) return 1

		// Then sort by completion percentage (highest first)
		const aCompletion = a.completionPercentage ?? 0
		const bCompletion = b.completionPercentage ?? 0

		if (aCompletion !== bCompletion) {
			return bCompletion - aCompletion
		}

		// If completion percentages are equal, sort alphabetically by display name
		return a.displayName.localeCompare(b.displayName)
	})
})

const $query = ref('')

const $fuse = computed(() => {
	if (!$locales.value || $locales.value.length === 0) return null
	return new Fuse($locales.value, {
		keys: [
			{
				name: 'displayName',
				weight: 2,
			},
			{
				name: 'translatedName',
				weight: 2,
			},
			{
				name: 'tag',
				weight: 1,
			},
		],
		includeScore: true,
		threshold: 0.4,
	})
})

const $searchResults = computed(() => {
	if (!$query.value || !$fuse.value) return null

	const results = $fuse.value.search($query.value)

	const autoLocale = $locales.value.find((locale) => locale.auto)
	const autoLocaleMatches =
		autoLocale &&
		formatMessage(messages.automaticLocale).toLowerCase().includes($query.value.toLowerCase())

	const searchResults = results.map((result) => result.item)

	if (autoLocaleMatches && autoLocale) {
		return [autoLocale, ...searchResults]
	}

	return searchResults
})

const $displayedLocales = computed(() => {
	if (!$query.value || !$searchResults.value) return $locales.value
	return $searchResults.value
})

const $changingTo = ref<string | undefined>()

const isChanging = () => $changingTo.value != null

const $failedLocale = ref<string>()

const $activeLocale = computed(() => {
	if ($changingTo.value != null) return $changingTo.value
	return vintl.automatic ? 'auto' : vintl.locale
})

async function changeLocale(value: string) {
	if ($activeLocale.value === value) return

	$changingTo.value = value

	try {
		await vintl.changeLocale(value)
		$failedLocale.value = undefined
	} catch {
		$failedLocale.value = value
	} finally {
		$changingTo.value = undefined
	}
}

function onItemKeydown(e: KeyboardEvent, locale: Locale) {
	switch (e.key) {
		case 'Enter':
		case ' ':
			break
		default:
			return
	}

	if (isModifierKeyDown(e) || isChanging()) return

	changeLocale(locale.tag)
}

function onItemClick(e: MouseEvent, locale: Locale) {
	if (isModifierKeyDown(e) || isChanging()) return

	changeLocale(locale.tag)
}

function getItemLabel(locale: Locale) {
	const label = locale.auto
		? formatMessage(messages.automaticLocale)
		: `${locale.translatedName}. ${locale.displayName}`

	if ($changingTo.value === locale.tag) {
		return formatMessage(messages.languageLabelApplying, { label })
	}

	if ($failedLocale.value === locale.tag) {
		return formatMessage(messages.languageLabelError, { label })
	}

	return label
}
</script>

<template>
	<div>
		<section class="universal-card">
			<h2 class="text-2xl">{{ formatMessage(commonSettingsMessages.language) }}</h2>

			<Admonition type="warning">
				{{ formatMessage(messages.languageWarning) }}
			</Admonition>

			<div class="card-description mt-4">
				<IntlFormatted :message-id="messages.languagesDescription">
					<template #crowdin-link="{ children }">
						<a href="https://translate.modrinth.com">
							<component :is="() => children" />
						</a>
					</template>
				</IntlFormatted>
			</div>

			<div class="search-container">
				<div class="iconified-input">
					<SearchIcon aria-hidden="true" class="text-lg" />
					<input
						v-model="$query"
						class="h-[40px]"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.searchPlaceholder)"
						:disabled="isChanging()"
					/>
					<Button v-if="$query" class="r-btn" @click="() => ($query = '')">
						<XIcon />
					</Button>
				</div>
			</div>

			<div class="languages-list">
				<template v-for="locale in $displayedLocales" :key="locale.tag">
					<div
						role="button"
						:aria-pressed="$activeLocale === locale.tag"
						:class="{
							'language-item': true,
							pending: $changingTo == locale.tag,
							errored: $failedLocale == locale.tag,
						}"
						:aria-describedby="
							$failedLocale == locale.tag ? `language__${locale.tag}__fail` : undefined
						"
						:aria-disabled="isChanging() && $changingTo !== locale.tag"
						:tabindex="0"
						:aria-label="getItemLabel(locale)"
						@click="(e) => onItemClick(e, locale)"
						@keydown="(e) => onItemKeydown(e, locale)"
					>
						<RadioButtonCheckedIcon v-if="$activeLocale === locale.tag" class="radio" />
						<RadioButtonIcon v-else class="radio" />

						<div class="language-names">
							<div class="language-name flex gap-2">
								{{ locale.auto ? formatMessage(messages.automaticLocale) : locale.translatedName }}
								<span
									v-if="!locale.auto && locale.isRTL"
									class="my-auto rounded-full bg-brand-highlight px-2 text-sm font-bold text-brand"
								>
									{{ formatMessage(messages.experimentalBadge) }}
								</span>
								<span
									v-if="!locale.auto && locale.completionPercentage !== undefined"
									class="bg-highlight-gray my-auto rounded-full px-2 text-sm font-medium text-contrast"
									:class="{
										'bg-highlight-green !text-brand-green': locale.completionPercentage >= 80,
										'bg-highlight-orange !text-brand-orange':
											locale.completionPercentage >= 45 && locale.completionPercentage < 80,
										'bg-highlight-red !text-brand-red': locale.completionPercentage < 45,
									}"
								>
									{{ locale.completionPercentage }}%
								</span>
							</div>
							<div v-if="!locale.auto" class="language-translated-name">
								{{ locale.displayName }}
							</div>
						</div>
					</div>

					<div
						v-if="$failedLocale === locale.tag"
						:id="`language__${locale.tag}__fail`"
						class="language-load-error"
					>
						<IssuesIcon />
						{{ formatMessage(messages.loadFailed) }}
					</div>
				</template>
			</div>
		</section>
	</div>
</template>

<style scoped lang="scss">
.languages-list {
	display: flex;
	flex-direction: column;
	gap: 0.6rem;
}

.language-item {
	display: flex;
	align-items: center;
	column-gap: 0.5rem;
	border: 0.15rem solid transparent;
	border-radius: var(--spacing-card-md);
	background: var(--color-button-bg);
	padding: var(--spacing-card-md);
	cursor: pointer;
	position: relative;
	overflow: hidden;

	&:not([aria-disabled='true']):hover {
		border-color: var(--color-button-bg-hover);
	}

	&:focus-visible,
	&:has(:focus-visible) {
		outline: 2px solid var(--color-brand);
	}

	&.errored {
		border-color: var(--color-red);

		&:hover {
			border-color: var(--color-red);
		}
	}

	&.pending::after {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;

		background-image: linear-gradient(
			102deg,
			rgba(0, 0, 0, 0) 0%,
			rgba(0, 0, 0, 0) 20%,
			rgba(0, 0, 0, 0.1) 45%,
			rgba(0, 0, 0, 0.1) 50%,
			rgba(0, 0, 0, 0) 80%,
			rgba(0, 0, 0, 0) 100%
		);

		background-repeat: no-repeat;
		animation: shimmerSliding 2.5s ease-out infinite;

		.dark-mode &,
		.oled-mode & {
			background-image: linear-gradient(
				102deg,
				rgba(255, 255, 255, 0) 0%,
				rgba(255, 255, 255, 0) 20%,
				rgba(255, 255, 255, 0.1) 45%,
				rgba(255, 255, 255, 0.1) 50%,
				rgba(255, 255, 255, 0) 80%,
				rgba(255, 255, 255, 0) 100%
			);
		}

		@keyframes shimmerSliding {
			from {
				left: -100%;
			}
			to {
				left: 100%;
			}
		}
	}

	&[aria-disabled='true']:not(.pending) {
		opacity: 0.8;
		pointer-events: none;
		cursor: default;
	}
}

.language-load-error {
	color: var(--color-red);
	font-size: var(--font-size-sm);
	margin-left: 0.3rem;
	display: flex;
	align-items: center;
	gap: 0.3rem;
}

.radio {
	width: 24px;
	height: 24px;
}

.language-names {
	display: flex;
	justify-content: space-between;
	flex: 1;
	flex-wrap: wrap;
}

.language-name {
	font-weight: bold;
}

.language-translated-name {
	color: var(--color-text-secondary);
}

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

.search-container {
	margin-bottom: var(--spacing-card-md);
}
</style>
