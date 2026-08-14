<script setup lang="ts">
import IntlMessageFormat, { type FormatXMLElementFn, type PrimitiveType } from 'intl-messageformat'
import { computed, defineComponent, markRaw, type PropType, useSlots, type VNode } from 'vue'

import type { MessageDescriptor } from '../../composables/i18n'
import { injectI18nDebug } from '../../composables/i18n-debug'
import { injectI18n } from '../../providers/i18n'

const VNodeRenderer = defineComponent({
	props: {
		node: { type: Object as PropType<VNode>, required: true },
	},
	setup(props) {
		return () => props.node
	},
})

const props = defineProps<{
	messageId: MessageDescriptor
	values?: Record<string, PrimitiveType>
}>()

const slots = useSlots()
const { t, locale } = injectI18n()
const debugContext = injectI18nDebug()

const debugEnabled = computed(() => debugContext?.enabled.value ?? false)
const debugKeyReveal = computed(() => debugContext?.keyReveal.value ?? false)

const formattedParts = computed(() => {
	const key = props.messageId.id
	const translation = t(key, {}) as string

	let msg: string
	if (translation && translation !== key) {
		msg = translation
	} else {
		msg = props.messageId.defaultMessage ?? key
	}

	if (debugEnabled.value) {
		debugContext!.registry.set(key, {
			key,
			value: msg,
			defaultMessage: props.messageId.defaultMessage,
			timestamp: Date.now(),
		})
		if (debugKeyReveal.value) {
			return [`\u300C${key}\u300D`]
		}
	}

	const slotHandlers: Record<string, FormatXMLElementFn<VNode>> = {}
	const slotNames = Object.keys(slots)

	for (const slotName of slotNames) {
		const normalizedName = slotName.startsWith('~') ? slotName.slice(1) : slotName
		slotHandlers[normalizedName] = (chunks) => {
			const slot = slots[slotName]
			if (slot) {
				const nodes = slot({
					children: chunks,
				})
				if (Array.isArray(nodes) && nodes.length === 1) {
					return markRaw(nodes[0]) as VNode
				}
				return markRaw(nodes) as VNode[]
			}
			return markRaw(chunks) as VNode[]
		}

		msg = msg.replace(
			new RegExp(`\\{${normalizedName}\\}`, 'g'),
			`<${normalizedName}></${normalizedName}>`,
		)
	}

	try {
		const formatter = new IntlMessageFormat(msg, locale.value)
		const result = formatter.format({
			...props.values,
			...slotHandlers,
		})

		return toFormattedParts(result)
	} catch {
		return [msg]
	}
})

function toFormattedParts(value: unknown): unknown[] {
	if (Array.isArray(value)) {
		return value.flatMap(toFormattedParts)
	}
	if (typeof value === 'object' && value !== null) {
		return [markRaw(value)]
	}
	return [value]
}

function isVNodePart(part: unknown): part is VNode {
	return typeof part === 'object' && part !== null
}
</script>

<template>
	<span
		style="display: contents"
		:data-i18n-key="debugEnabled && !debugKeyReveal ? messageId.id : undefined"
	>
		<template v-for="(part, index) in formattedParts" :key="index">
			<VNodeRenderer v-if="isVNodePart(part)" :node="part" />
			<template v-else>{{ part }}</template>
		</template>
	</span>
</template>
