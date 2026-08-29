<template>
	<Suspense>
		<template #default>
			<component
				:is="resolvedTag"
				class="markdown-body"
				:class="{ basic: variant === 'basic' }"
				v-bind="$attrs"
			>
				<Markdown
					:value="source"
					:unwrap="unwrap"
					:plugins="plugins"
					:components="components"
					:options="{ registerDefaultPlugins: false }"
				/>
			</component>
		</template>
		<template #fallback>
			<component
				:is="resolvedTag"
				class="markdown-body"
				:class="{ basic: variant === 'basic' }"
				v-bind="$attrs"
			/>
		</template>
	</Suspense>
</template>

<script setup lang="ts">
import { Markdown } from '@comark/vue'
import { modrinthBasicPlugins, modrinthForceLinkTarget, modrinthPlugins } from '@modrinth/utils'
import { computed } from 'vue'

import MarkdownAlert, { alertMarkerTypes } from './markdown/MarkdownAlert.vue'
import MarkdownCollectionEmbed from './markdown/MarkdownCollectionEmbed.vue'
import MarkdownHighlightedPre from './markdown/MarkdownHighlightedPre.vue'
import MarkdownOrganizationEmbed from './markdown/MarkdownOrganizationEmbed.vue'
import MarkdownProjectEmbed from './markdown/MarkdownProjectEmbed.vue'
import MarkdownTaskCheckbox from './markdown/MarkdownTaskCheckbox.vue'
import MarkdownUserEmbed from './markdown/MarkdownUserEmbed.vue'

defineOptions({
	inheritAttrs: false,
})

const props = withDefaults(
	defineProps<{
		source: string
		variant?: 'full' | 'basic'
		highlight?: boolean
		unwrapParagraph?: boolean
		target?: string
		tag?: string
	}>(),
	{
		variant: 'full',
		highlight: false,
		unwrapParagraph: false,
		target: undefined,
		tag: undefined,
	},
)

const resolvedTag = computed(() => props.tag ?? (props.variant === 'basic' ? 'span' : 'div'))
const unwrap = computed(() => (props.variant === 'basic' ? 'p' : props.unwrapParagraph))

const plugins = computed(() => {
	if (props.variant !== 'basic') return modrinthPlugins
	return props.target
		? [...modrinthBasicPlugins, modrinthForceLinkTarget(props.target)]
		: modrinthBasicPlugins
})

const alertComponents = Object.fromEntries(alertMarkerTypes.map((marker) => [marker, MarkdownAlert]))

const components = computed(() =>
	props.variant === 'basic'
		? {}
		: {
				...alertComponents,
				project: MarkdownProjectEmbed,
				user: MarkdownUserEmbed,
				organization: MarkdownOrganizationEmbed,
				collection: MarkdownCollectionEmbed,
				input: MarkdownTaskCheckbox,
				...(props.highlight ? { pre: MarkdownHighlightedPre } : {}),
			},
)
</script>

<style scoped>
.markdown-body.basic {
	display: block;
	max-width: 100%;
	overflow-wrap: break-word;
}

.markdown-body.basic :deep(a) {
	color: var(--color-link);
}

.markdown-body.basic :deep(a:hover) {
	text-decoration: underline;
}

.markdown-body.basic :deep(code) {
	background-color: var(--surface-4);
	font-size: 12px;
	padding: 2px 4px;
	border-radius: 4px;
}
</style>
