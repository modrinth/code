<script setup lang="ts">
import IntlMessageFormat, { type FormatXMLElementFn, type PrimitiveType } from 'intl-messageformat'
import { computed, useSlots, type VNode } from 'vue'
import { useI18n } from 'vue-i18n'

import type { MessageDescriptor } from '@/utils/i18n-vintl'

const props = defineProps<{
	messageId: MessageDescriptor
	values?: Record<string, PrimitiveType>
}>()

const slots = useSlots()
const { t, locale, messages } = useI18n()

const formattedParts = computed(() => {
	const key = props.messageId.id
	const localeMessages = messages.value[locale.value] as Record<string, string> | undefined
	const translation = localeMessages?.[key]

	let msg: string
	if (translation && translation !== key) {
		msg = translation
	} else {
		const translatedT = t(key, {})
		if (translatedT && translatedT !== key) {
			msg = translatedT
		} else {
			msg = props.messageId.defaultMessage ?? key
		}
	}

	const slotHandlers: Record<string, FormatXMLElementFn<VNode>> = {}
	const slotNames = Object.keys(slots)

	for (const slotName of slotNames) {
		slotHandlers[slotName] = (chunks) => {
			const slot = slots[slotName]
			if (slot) {
				return slot({
					children: chunks,
				})
			}
			return chunks as VNode[]
		}
	}

	try {
		const formatter = new IntlMessageFormat(msg, locale.value)
		const result = formatter.format({
			...props.values,
			...slotHandlers,
		})

		if (Array.isArray(result)) {
			return result
		}
		return [result]
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
