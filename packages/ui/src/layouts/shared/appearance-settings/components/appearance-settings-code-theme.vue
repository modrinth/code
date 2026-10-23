<template>
	<div class="flex flex-col gap-4">
		<Combobox
			class="w-full sm:w-80"
			searchable
			:aria-label="formatMessage(messages.codeThemeTitle)"
			:search-placeholder="formatMessage(messages.codeThemeSearch)"
			:model-value="theme"
			:options="options"
			@update:model-value="(value) => value && (theme = value)"
		/>
		<pre
			class="hljs m-0 overflow-x-auto rounded-2xl p-4 text-sm [tab-size:4]"
			:aria-label="formatMessage(messages.codeThemePreview)"
		><code v-html="preview" /></pre>
	</div>
</template>

<script setup lang="ts">
import { hljs } from '@modrinth/utils/highlightjs/index'
import { computed } from 'vue'

import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import { DEFAULT_CODE_THEME, useCodeHighlightTheme } from '#ui/composables/code-highlight-theme'
import { useVIntl } from '#ui/composables/i18n'

import { appearanceSettingsMessages as messages } from '../messages'

/** Picks the theme code is highlighted with, previewing it on a short sample. */
const { formatMessage } = useVIntl()
const { theme, themes } = useCodeHighlightTheme()

const SAMPLE = `// Registers a listener once the mod loads
public final class ExampleMod implements ModInitializer {
	private static final int MAX_ITEMS = 64;

	@Override
	public void onInitialize() {
		String name = "example-mod";
		if (MAX_ITEMS > 0 && !name.isEmpty()) {
			LOGGER.info("Loaded {} with {} items", name, MAX_ITEMS);
		}
	}
}`

const preview = hljs.highlight(SAMPLE, { language: 'java' }).value

const options = computed<ComboboxOption<string>[]>(() => [
	{ value: DEFAULT_CODE_THEME, label: formatMessage(messages.codeThemeDefault) },
	...themes.map((entry) => ({ value: entry.id, label: entry.label })),
])
</script>
