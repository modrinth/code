<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import Fuse from 'fuse.js/dist/fuse.basic'
import { computed, onMounted, ref } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import {
	buildLocaleMessages,
	defineMessages,
	type LocaleDefinition,
	useVIntl,
} from '#ui/composables/i18n'
import { metaLocaleModules } from '#ui/locales.ts'
import { isModifierKeyDown } from '#ui/utils/events'

import type { LanguageCoverageStats } from './language-settings-coverage'

const { formatMessage } = useVIntl()

const props = defineProps<{
	product: 'app' | 'website'
	currentLocale: string
	locales: LocaleDefinition[]
	onLocaleChange: (locale: string) => void | Promise<void>
	isChanging?: boolean
	coverageByLocale?: Record<string, LanguageCoverageStats>
}>()

const messages = defineMessages({
	noResults: {
		id: 'settings.language.languages.search.no-results',
		defaultMessage: 'No languages match your search.',
	},
	searchFieldPlaceholder: {
		id: 'settings.language.languages.search-field.placeholder',
		defaultMessage: 'Search for a language...',
	},
	searchResultsAnnouncement: {
		id: 'settings.language.languages.search-results-announcement',
		defaultMessage:
			'{matches, plural, =0 {No languages match} one {# language matches} other {# languages match}} your search.',
	},
	standardLanguages: {
		id: 'settings.language.categories.default',
		defaultMessage: 'Standard languages',
	},
	searchResults: {
		id: 'settings.language.categories.search-result',
		defaultMessage: 'Search results',
	},
	coverageLabel: {
		id: 'settings.language.coverage.label',
		defaultMessage: '{percentage}% supported',
	},
	appCoverageTooltip: {
		id: 'settings.language.coverage.app-tooltip',
		defaultMessage: 'About {percentage}% of the Modrinth App is available in this language.',
	},
	websiteCoverageTooltip: {
		id: 'settings.language.coverage.website-tooltip',
		defaultMessage: 'About {percentage}% of the website is available in this language.',
	},
})

const localeMetas = buildLocaleMessages(metaLocaleModules)

type Category = 'default' | 'searchResult'

type LocaleInfo = {
	category: Category
	tag: string
	displayName: string
	browserDisplayName: string
	flagUrl?: string
	searchTerms?: string[]
	coverage?: LanguageCoverageStats
}

const localeFlagRegions: Record<string, string> = {
	'es-419': 'mx',
	'sr-CS': 'rs',
}

const $browserLocales = ref([props.currentLocale])

onMounted(() => {
	$browserLocales.value = navigator.languages.length
		? [...navigator.languages]
		: [navigator.language]
})

const $browserDisplayNames = computed(() => {
	try {
		return new Intl.DisplayNames($browserLocales.value, { type: 'language' })
	} catch {
		return undefined
	}
})

function getBrowserDisplayName(tag: string, fallback: string): string {
	try {
		return $browserDisplayNames.value?.of(tag) ?? fallback
	} catch {
		try {
			return $browserDisplayNames.value?.of(tag.split('-')[0]) ?? fallback
		} catch {
			return fallback
		}
	}
}

function getFlagUrl(tag: string): string | undefined {
	const region = localeFlagRegions[tag] ?? tag.split('-').at(-1)
	if (!region || !/^[a-z]{2}$/i.test(region)) return undefined

	return `https://flagcdn.com/${region.toLowerCase()}.svg`
}

const $locales = computed(() => {
	const result: LocaleInfo[] = []

	for (const loc of props.locales) {
		const tag = loc.code
		const meta = localeMetas[tag] ?? null
		const displayName = meta?.displayName ?? loc.name
		const translatedName = formatMessage(loc.translatedName)
		const browserDisplayName = getBrowserDisplayName(tag, translatedName)
		const searchTerms = meta?.searchTerms === '-' ? undefined : meta?.searchTerms?.split('\n')

		result.push({
			tag,
			category: 'default',
			displayName,
			browserDisplayName,
			flagUrl: getFlagUrl(tag),
			searchTerms,
			coverage: props.coverageByLocale?.[tag],
		})
	}

	return result.sort((a, b) => (b.coverage?.percentage ?? -1) - (a.coverage?.percentage ?? -1))
})

const $query = ref('')

const isQueryEmpty = () => $query.value.trim().length === 0

const fuse = computed(
	() =>
		new Fuse<LocaleInfo>($locales.value, {
			keys: ['tag', 'displayName', 'browserDisplayName', 'searchTerms'],
			threshold: 0.4,
			distance: 100,
		}),
)

const $categories = computed(() => {
	const categories = new Map<Category, LocaleInfo[]>()
	categories.set('default', $locales.value)
	return categories
})

const $searchResults = computed(() => {
	return new Map<Category, LocaleInfo[]>([
		['searchResult', isQueryEmpty() ? [] : fuse.value.search($query.value).map(({ item }) => item)],
	])
})

const $displayCategories = computed(() =>
	isQueryEmpty() ? $categories.value : $searchResults.value,
)

const $changingTo = ref<string | undefined>()

const isChangingLocale = () => $changingTo.value != null || props.isChanging

const $activeLocale = computed(() => {
	if ($changingTo.value != null) return $changingTo.value
	return props.currentLocale
})

function changeLocale(value: string) {
	if ($activeLocale.value === value) return

	const result = props.onLocaleChange(value)
	if (!result) return

	$changingTo.value = value
	void result.then(
		() => {
			$changingTo.value = undefined
		},
		() => {
			$changingTo.value = undefined
		},
	)
}

const $languagesList = ref<HTMLDivElement | undefined>()

function onSearchKeydown(e: KeyboardEvent) {
	if (e.key !== 'Enter' || isModifierKeyDown(e)) return

	const focusableTarget = $languagesList.value?.querySelector(
		'button:not(:disabled), [tabindex]:not([tabindex="-1"])',
	) as HTMLElement | undefined

	focusableTarget?.focus()
}

function onItemClick(e: MouseEvent, loc: LocaleInfo) {
	if (isModifierKeyDown(e) || isChangingLocale()) return

	changeLocale(loc.tag)
}

function showBrowserDisplayName(loc: LocaleInfo): boolean {
	return (
		loc.browserDisplayName.localeCompare(loc.displayName, undefined, { sensitivity: 'base' }) !== 0
	)
}

function getItemLabel(loc: LocaleInfo) {
	const coverageLabel = loc.coverage
		? `. ${formatMessage(messages.coverageLabel, { percentage: loc.coverage.percentage })}`
		: ''
	const browserDisplayName = showBrowserDisplayName(loc) ? `. ${loc.browserDisplayName}` : ''
	return `${loc.displayName}${browserDisplayName}${coverageLabel}`
}

function getCoverageTooltip(coverage: LanguageCoverageStats): string {
	const message =
		props.product === 'app' ? messages.appCoverageTooltip : messages.websiteCoverageTooltip

	return formatMessage(message, {
		percentage: coverage.percentage,
	})
}

function getCategoryName(category: Category): string {
	if (category === 'searchResult') {
		return formatMessage(messages.searchResults)
	}
	return formatMessage(messages.standardLanguages)
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<div v-if="$locales.length > 1" class="-mb-4">
			<StyledInput
				id="language-search"
				v-model="$query"
				:icon="SearchIcon"
				name="language"
				type="search"
				:placeholder="formatMessage(messages.searchFieldPlaceholder)"
				:disabled="isChangingLocale()"
				wrapper-class="w-full"
				@keydown="onSearchKeydown"
			/>

			<div id="language-search-results-announcements" class="visually-hidden" aria-live="polite">
				{{
					isQueryEmpty()
						? ''
						: formatMessage(messages.searchResultsAnnouncement, {
								matches: $searchResults.get('searchResult')?.length ?? 0,
							})
				}}
			</div>
		</div>

		<div ref="$languagesList" class="flex flex-col gap-2.5">
			<template v-for="[category, categoryLocales] in $displayCategories" :key="category">
				<strong class="mt-4 font-semibold text-contrast">
					{{ getCategoryName(category) }}
				</strong>

				<div
					v-if="category === 'searchResult' && categoryLocales.length === 0"
					class="p-4 text-secondary"
					tabindex="0"
				>
					{{ formatMessage(messages.noResults) }}
				</div>

				<template v-for="loc in categoryLocales" :key="loc.tag">
					<Button
						:type="$activeLocale === loc.tag ? 'colored' : 'base'"
						:color="$activeLocale === loc.tag ? 'green' : undefined"
						:aria-pressed="$activeLocale === loc.tag"
						:disabled="isChangingLocale() && $changingTo !== loc.tag"
						:aria-label="getItemLabel(loc)"
						class="w-full !justify-start !gap-2 !text-left sm:!h-10"
						:class="
							$activeLocale === loc.tag
								? '!bg-[var(--color-button-bg-selected)] !text-[var(--color-button-text-selected)]'
								: ''
						"
						@click="(e) => onItemClick(e, loc)"
					>
						<img
							v-if="loc.flagUrl"
							:src="loc.flagUrl"
							alt=""
							aria-hidden="true"
							class="h-4 w-6 shrink-0 rounded-sm object-cover"
							loading="lazy"
						/>

						<span class="flex min-w-0 flex-1 items-baseline gap-2 overflow-hidden">
							<span class="truncate text-sm sm:text-base">{{ loc.displayName }}</span>
							<span
								v-if="showBrowserDisplayName(loc)"
								class="truncate text-xs font-normal text-secondary sm:text-sm"
							>
								{{ loc.browserDisplayName }}
							</span>
						</span>

						<span
							v-if="loc.coverage"
							v-tooltip="getCoverageTooltip(loc.coverage)"
							class="ml-auto shrink-0 text-xs font-normal text-secondary sm:text-sm"
						>
							{{ loc.coverage.percentage }}%
						</span>
					</Button>
				</template>
			</template>
		</div>
	</div>
</template>
