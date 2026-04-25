<template>
	<span
		v-if="channel === 'ALPHA'"
		class="rounded-full bg-bg-red px-2 text-sm font-bold text-red"
		:class="{ 'shrink-0': affix }"
	>
		{{ formatMessage(commonMessages.alpha) }}
	</span>
	<span
		v-else-if="channel === 'BETA'"
		class="rounded-full bg-bg-orange px-2 text-sm font-bold text-orange"
		:class="{ 'shrink-0': affix }"
	>
		{{ formatMessage(commonMessages.beta) }}
	</span>
</template>

<script setup lang="ts">
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

defineProps<{
	channel: 'ALPHA' | 'BETA' | null | undefined
	/** When true, prevents the badge from shrinking in flex rows (e.g. search field affix). */
	affix?: boolean
}>()

const { formatMessage } = useVIntl()
</script>
