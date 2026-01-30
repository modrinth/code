<script setup lang="ts">
import IntlMessageFormat, { type FormatXMLElementFn, type PrimitiveType } from 'intl-messageformat'
import { computed, markRaw, useSlots, type VNode } from 'vue'

import type { MessageDescriptor } from '../../composables/i18n'
import { injectI18n } from '../../providers/i18n'

const props = defineProps<{
	messageId: MessageDescriptor
	values?: Record<string, PrimitiveType>
}>()

const slots = useSlots()
const { t, locale } = injectI18n()

const formattedParts = computed(() => {
	const key = props.messageId.id
	const translation = t(key, {}) as string

	let msg: string
	if (translation && translation !== key) {
		msg = translation
	} else {
		msg = props.messageId.defaultMessage ?? key
	}

	const slotHandlers: Record<string, FormatXMLElementFn<VNode>> = {}
	const slotNames = Object.keys(slots)

	for (const slotName of slotNames) {
		const normalizedName = slotName.startsWith('~') ? slotName.slice(1) : slotName
		slotHandlers[normalizedName] = (chunks) => {
			const slot = slots[slotName]
			if (slot) {
				return markRaw(
					slot({
						children: chunks,
					}),
				) as VNode[]
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

		// ensure result array items are marked as raw if they're VNodes
		// prevents VNodes from entering the reactive system and SSR payload
		if (Array.isArray(result)) {
			return result.map((part) =>
				typeof part === 'object' && part !== null ? markRaw(part) : part,
			)
		}
		return [typeof result === 'object' && result !== null ? markRaw(result) : result]
	} catch {
		return [msg]
	}
})
</script>

<template>
	<template v-for="(part, index) in formattedParts" :key="index">
		<component :is="() => part" v-if="typeof part === 'object'" />
		<template v-else>{{ part }}</template>
	</template>
</template>
