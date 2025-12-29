<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon, SearchIcon } from '@modrinth/assets'
import Fuse from 'fuse.js/dist/fuse.basic'
import { computed, ref, watchSyncEffect } from 'vue'

import { defineMessages, type LocaleDefinition, useVIntl } from '../../composables/i18n'
import { isModifierKeyDown } from '../../utils/events'

const { formatMessage } = useVIntl()

const props = defineProps<{
	currentLocale: string
	locales: LocaleDefinition[]
	onLocaleChange: (locale: string) => Promise<void>
	isChanging?: boolean
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
})

type Category = 'default' | 'searchResult'

type LocaleInfo = {
	category: Category
	tag: string
	displayName: string
	nativeName: string
	searchTerms?: string[]
}

const displayNames = new Intl.DisplayNames(['en'], { type: 'language' })

const $locales = computed(() => {
	const result: LocaleInfo[] = []

	for (const loc of props.locales) {
		const tag = loc.code
		const name = loc.name || displayNames.of(tag) || tag

		const nativeDisplayNames = new Intl.DisplayNames([tag], { type: 'language' })
		const nativeName = nativeDisplayNames.of(tag) || tag

		result.push({
			tag,
			category: 'default',
			displayName: name,
			nativeName,
			searchTerms: [tag, name, nativeName],
		})
	}

	return result
})

const $query = ref('')

const isQueryEmpty = () => $query.value.trim().length === 0

const fuse = new Fuse<LocaleInfo>([], {
	keys: ['tag', 'displayName', 'nativeName', 'searchTerms'],
	threshold: 0.4,
	distance: 100,
})

watchSyncEffect(() => fuse.setCollection($locales.value))

const $categories = computed(() => {
	const categories = new Map<Category, LocaleInfo[]>()
	categories.set('default', $locales.value)
	return categories
})

const $searchResults = computed(() => {
	return new Map<Category, LocaleInfo[]>([
		['searchResult', isQueryEmpty() ? [] : fuse.search($query.value).map(({ item }) => item)],
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

async function changeLocale(value: string) {
	if ($activeLocale.value === value) return

	$changingTo.value = value

	try {
		await props.onLocaleChange(value)
	} finally {
		$changingTo.value = undefined
	}
}

const $languagesList = ref<HTMLDivElement | undefined>()

function onSearchKeydown(e: KeyboardEvent) {
	if (e.key !== 'Enter' || isModifierKeyDown(e)) return

	const focusableTarget = $languagesList.value?.querySelector(
		'input, [tabindex]:not([tabindex="-1"])',
	) as HTMLElement | undefined

	focusableTarget?.focus()
}

function onItemKeydown(e: KeyboardEvent, loc: LocaleInfo) {
	switch (e.key) {
		case 'Enter':
		case ' ':
			break
		default:
			return
	}

	if (isModifierKeyDown(e) || isChangingLocale()) return

	changeLocale(loc.tag)
}

function onItemClick(e: MouseEvent, loc: LocaleInfo) {
	if (isModifierKeyDown(e) || isChangingLocale()) return

	changeLocale(loc.tag)
}

function getItemLabel(loc: LocaleInfo) {
	return `${loc.nativeName}. ${loc.displayName}`
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
		<div v-if="$locales.length > 1" class="iconified-input w-full -mb-4">
			<SearchIcon />
			<input
				id="language-search"
				v-model="$query"
				name="language"
				type="search"
				:placeholder="formatMessage(messages.searchFieldPlaceholder)"
				class="input-text-inherit"
				:disabled="isChangingLocale()"
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
				<strong class="mt-4 font-bold">
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
					<div
						role="button"
						:aria-pressed="$activeLocale === loc.tag"
						:class="[
							'flex items-center gap-2 border-2 rounded-lg bg-surface-4 p-4 py-2 cursor-pointer relative overflow-hidden border-transparent transition-colors duration-100',
							'focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand hover:border-surface-5 border-solid',
							isChangingLocale() && $changingTo !== loc.tag
								? 'opacity-80 pointer-events-none cursor-default'
								: '',
						]"
						:aria-disabled="isChangingLocale() && $changingTo !== loc.tag"
						:tabindex="0"
						:aria-label="getItemLabel(loc)"
						@click="(e) => onItemClick(e, loc)"
						@keydown="(e) => onItemKeydown(e, loc)"
					>
						<RadioButtonCheckedIcon v-if="$activeLocale === loc.tag" class="size-6" />
						<RadioButtonIcon v-else class="size-6" />

						<div class="flex flex-1 flex-wrap justify-between">
							<div class="font-bold">
								{{ loc.displayName }}
							</div>

							<div>
								{{ loc.nativeName }}
							</div>
						</div>
					</div>
				</template>
			</template>
		</div>
	</div>
</template>
