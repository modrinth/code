<script setup lang="ts">
import Fuse from 'fuse.js/dist/fuse.basic'
import RadioButtonIcon from '~/assets/images/utils/radio-button.svg?component'
import RadioButtonCheckedIcon from '~/assets/images/utils/radio-button-checked.svg?component'
import WarningIcon from '~/assets/images/utils/issues.svg?component'
import { isModifierKeyDown } from '~/helpers/events.ts'
import { commonSettingsMessages } from '~/utils/common-messages.ts'

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
  noResults: {
    id: 'settings.language.languages.search.no-results',
    defaultMessage: 'No languages match your search.',
  },
  searchFieldDescription: {
    id: 'settings.language.languages.search-field.description',
    defaultMessage: 'Submit to focus the first search result',
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
})

const categoryNames = defineMessages({
  auto: {
    id: 'settings.language.categories.auto',
    defaultMessage: 'Automatic',
  },
  default: {
    id: 'settings.language.categories.default',
    defaultMessage: 'Standard languages',
  },
  fun: {
    id: 'settings.language.categories.fun',
    defaultMessage: 'Fun languages',
  },
  experimental: {
    id: 'settings.language.categories.experimental',
    defaultMessage: 'Experimental languages',
  },
  searchResult: {
    id: 'settings.language.categories.search-result',
    defaultMessage: 'Search results',
  },
})

type Category = keyof typeof categoryNames

const categoryOrder: Category[] = ['auto', 'default', 'fun', 'experimental']

function normalizeCategoryName(name?: string): keyof typeof categoryNames {
  switch (name) {
    case 'auto':
    case 'fun':
    case 'experimental':
      return name
    default:
      return 'default'
  }
}

type LocaleBase = {
  category: Category
  tag: string
  searchTerms?: string[]
}

type AutomaticLocale = LocaleBase & {
  auto: true
}

type CommonLocale = LocaleBase & {
  auto?: never
  displayName: string
  defaultName: string
  translatedName: string
}

type Locale = AutomaticLocale | CommonLocale

const $defaultNames = useDisplayNames(() => vintl.defaultLocale)

const $translatedNames = useDisplayNames(() => vintl.locale)

const $locales = computed(() => {
  const locales: Locale[] = []

  locales.push({
    auto: true,
    tag: 'auto',
    category: 'auto',
    searchTerms: [
      'automatic',
      'Sync with the system language',
      formatMessage(messages.automaticLocale),
    ],
  })

  for (const locale of vintl.availableLocales) {
    let displayName = locale.meta?.displayName

    if (displayName == null) {
      displayName = createDisplayNames(locale.tag).of(locale.tag) ?? locale.tag
    }

    let defaultName = vintl.defaultResources['languages.json']?.[locale.tag]

    if (defaultName == null) {
      defaultName = $defaultNames.value.of(locale.tag) ?? locale.tag
    }

    let translatedName = vintl.resources['languages.json']?.[locale.tag]

    if (translatedName == null) {
      translatedName = $translatedNames.value.of(locale.tag) ?? locale.tag
    }

    let searchTerms = locale.meta?.searchTerms
    if (searchTerms === '-') searchTerms = undefined

    locales.push({
      tag: locale.tag,
      category: normalizeCategoryName(locale.meta?.category),
      displayName,
      defaultName,
      translatedName,
      searchTerms: searchTerms?.split('\n'),
    })
  }

  return locales
})

const $query = ref('')

const isQueryEmpty = () => $query.value.trim().length === 0

const fuse = new Fuse<Locale>([], {
  keys: ['tag', 'displayName', 'translatedName', 'englishName', 'searchTerms'],
  threshold: 0.4,
  distance: 100,
})

watchSyncEffect(() => fuse.setCollection($locales.value))

const $categories = computed(() => {
  const categories = new Map<Category, Locale[]>()

  for (const category of categoryOrder) categories.set(category, [])

  for (const locale of $locales.value) {
    let categoryLocales = categories.get(locale.category)

    if (categoryLocales == null) {
      categoryLocales = []
      categories.set(locale.category, categoryLocales)
    }

    categoryLocales.push(locale)
  }

  for (const categoryKey of [...categories.keys()]) {
    if (categories.get(categoryKey)?.length === 0) {
      categories.delete(categoryKey)
    }
  }

  return categories
})

const $searchResults = computed(() => {
  return new Map<Category, Locale[]>([
    ['searchResult', isQueryEmpty() ? [] : fuse.search($query.value).map(({ item }) => item)],
  ])
})

const $displayCategories = computed(() =>
  isQueryEmpty() ? $categories.value : $searchResults.value
)

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
  } catch (err) {
    $failedLocale.value = value
  } finally {
    $changingTo.value = undefined
  }
}

const $languagesList = ref<HTMLDivElement | undefined>()

function onSearchKeydown(e: KeyboardEvent) {
  if (e.key !== 'Enter' || isModifierKeyDown(e)) return

  const focusableTarget = $languagesList.value?.querySelector(
    'input, [tabindex]:not([tabindex="-1"])'
  ) as HTMLElement | undefined

  focusableTarget?.focus()
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
      <h2>{{ formatMessage(commonSettingsMessages.language) }}</h2>

      <div class="card-description">
        <IntlFormatted :message-id="messages.languagesDescription">
          <template #crowdin-link="{ children }">
            <a href="https://crowdin.com/project/modrinth">
              <component :is="() => children" />
            </a>
          </template>
        </IntlFormatted>
      </div>

      <div class="search-container">
        <input
          id="language-search"
          v-model="$query"
          name="language"
          type="search"
          :placeholder="formatMessage(messages.searchFieldPlaceholder)"
          class="language-search"
          aria-describedby="language-search-description"
          :disabled="isChanging()"
          @keydown="onSearchKeydown"
        />

        <div id="language-search-description" class="visually-hidden">
          {{ formatMessage(messages.searchFieldDescription) }}
        </div>

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
        <template v-for="[category, locales] in $displayCategories" :key="category">
          <strong class="category-name">
            {{ formatMessage(categoryNames[category]) }}
          </strong>

          <div
            v-if="category === 'searchResult' && locales.length === 0"
            class="no-results"
            tabindex="0"
          >
            {{ formatMessage(messages.noResults) }}
          </div>

          <template v-for="locale in locales" :key="locale.tag">
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
                <div class="language-name">
                  {{ locale.auto ? formatMessage(messages.automaticLocale) : locale.displayName }}
                </div>

                <div v-if="!locale.auto" class="language-translated-name">
                  {{ locale.translatedName }}
                </div>
              </div>
            </div>

            <div
              v-if="$failedLocale === locale.tag"
              :id="`language__${locale.tag}__fail`"
              class="language-load-error"
            >
              <WarningIcon /> {{ formatMessage(messages.loadFailed) }}
            </div>
          </template>
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

.language-search {
  width: 100%;
}

.search-container {
  margin-bottom: var(--spacing-card-md);
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

.category-name {
  margin-top: var(--spacing-card-md);
}
</style>
