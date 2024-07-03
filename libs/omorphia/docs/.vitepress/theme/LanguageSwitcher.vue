<script setup lang="ts">
import { useVIntl } from '@vintl/vintl'
import { DropdownSelect } from 'omorphia'
import { computed, ref } from 'vue'

const { $locales, $config, changeLocale } = useVIntl()

const getLocaleDisplayName = (() => {
  const cache = new Map<string, Intl.DisplayNames>()

  return function getLocaleDisplayName(locale: string) {
    let displayNames = cache.get(locale)
    if (displayNames == null) {
      displayNames = new Intl.DisplayNames(locale, {
        type: 'language',
        languageDisplay: 'standard',
      })
      cache.set(locale, displayNames)
    }
    return displayNames.of(locale)
  }
})()

const isChanging = ref(false)

const currentLocale = computed({
  get() {
    return $config.locale
  },
  async set(value) {
    if (isChanging.value) return

    try {
      isChanging.value = true
      await changeLocale(value)
    } finally {
      isChanging.value = false
    }
  },
})
</script>
<template>
  <div class="LanguageSwitcher">
    <h2 class="title">Playground language</h2>

    <DropdownSelect
      class="locale-dropdown"
      name="locale"
      v-model="currentLocale"
      placeholder="Change language"
      :disabled="isChanging"
      :options="Array.from($locales).map(([{ tag }]) => tag)"
      :display-name="(locale: string) => getLocaleDisplayName(locale)"
    />
  </div>
</template>
<style scoped>
.LanguageSwitcher {
  padding-block: 18px;
  border-bottom: 1px solid var(--vp-c-divider);
}

.LanguageSwitcher .title {
  font-weight: 700;
  font-size: 14px;
  color: var(--vp-c-text-1);
}

.LanguageSwitcher .locale-dropdown {
  width: 200px;
  font-size: 14px;
}
</style>
