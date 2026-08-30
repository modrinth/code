<template>
	<Suspense>
		<template #default>
			<component
				:is="resolvedTag"
				class="markdown-body"
				:class="{ basic: variant === 'basic' }"
				v-bind="$attrs"
			>
				<Markdown :value="parsed ?? source" :unwrap="unwrap" :plugins="plugins" :components="components" />
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
import 'katex/dist/katex.min.css'

import { Markdown } from '@comark/vue'
import { Math as MarkdownMath } from '@comark/vue/plugins/math'
import { basicPlugins, defaultPlugins, forceLinkTarget } from '@modrinth/utils'
import { createSerializedMarkdownParser } from 'comark'
import { computed } from 'vue'

import { useAsyncData } from '#app'

import MarkdownAlert from './markdown/MarkdownAlert.vue'
import MarkdownCollectionEmbed from './markdown/MarkdownCollectionEmbed.vue'
import MarkdownHighlightedPre from './markdown/MarkdownHighlightedPre.vue'
import MarkdownMermaid from './markdown/MarkdownMermaid.vue'
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
	if (props.variant !== 'basic') return defaultPlugins
	return props.target ? [...basicPlugins, forceLinkTarget(props.target)] : basicPlugins
})

const { data: parsed } = await useAsyncData(
	() => `markdown-body:${props.variant}:${unwrap.value}:${props.target ?? ''}:${props.source}`,
	() => createSerializedMarkdownParser({ ...(unwrap.value ? { unwrap: unwrap.value } : {}), plugins: plugins.value })(props.source),
	{ watch: [() => props.source, () => props.variant, unwrap, () => props.target] },
)

const components = computed(() =>
	props.variant === 'basic'
		? {}
		: {
				alert: MarkdownAlert,
				project: MarkdownProjectEmbed,
				user: MarkdownUserEmbed,
				organization: MarkdownOrganizationEmbed,
				collection: MarkdownCollectionEmbed,
				input: MarkdownTaskCheckbox,
				math: MarkdownMath,
				mermaid: MarkdownMermaid,
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
