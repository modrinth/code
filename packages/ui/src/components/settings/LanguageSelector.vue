<script setup lang="ts">
import { RadioButtonCheckedIcon, RadioButtonIcon } from '@modrinth/assets'
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
	<div class="language-selector">
		<div v-if="$locales.length > 1" class="search-container">
			<input
				id="language-search"
				v-model="$query"
				name="language"
				type="search"
				:placeholder="formatMessage(messages.searchFieldPlaceholder)"
				class="language-search"
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

		<div ref="$languagesList" class="languages-list">
			<template v-for="[category, categoryLocales] in $displayCategories" :key="category">
				<strong class="category-name">
					{{ getCategoryName(category) }}
				</strong>

				<div
					v-if="category === 'searchResult' && categoryLocales.length === 0"
					class="no-results"
					tabindex="0"
				>
					{{ formatMessage(messages.noResults) }}
				</div>

				<template v-for="loc in categoryLocales" :key="loc.tag">
					<div
						role="button"
						:aria-pressed="$activeLocale === loc.tag"
						:class="{
							'language-item': true,
							pending: $changingTo === loc.tag,
						}"
						:aria-disabled="isChangingLocale() && $changingTo !== loc.tag"
						:tabindex="0"
						:aria-label="getItemLabel(loc)"
						@click="(e) => onItemClick(e, loc)"
						@keydown="(e) => onItemKeydown(e, loc)"
					>
						<RadioButtonCheckedIcon v-if="$activeLocale === loc.tag" class="radio" />
						<RadioButtonIcon v-else class="radio" />

						<div class="language-names">
							<div class="language-name">
								{{ loc.displayName }}
							</div>

							<div class="language-translated-name">
								{{ loc.nativeName }}
							</div>
						</div>
					</div>
				</template>
			</template>
		</div>
	</div>
</template>

<style scoped lang="scss">
.language-selector {
	display: flex;
	flex-direction: column;
	gap: var(--spacing-card-md);
}

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

.language-search {
	width: 100%;
}

.search-container {
	margin-bottom: var(--spacing-card-md);
}

.category-name {
	margin-top: var(--spacing-card-md);
}

.no-results {
	padding: var(--spacing-card-md);
	color: var(--color-text-secondary);
}
</style>
