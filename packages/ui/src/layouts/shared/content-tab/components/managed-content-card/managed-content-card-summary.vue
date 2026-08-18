<script setup lang="ts">
import { BoxIcon, BracesIcon, GlassesIcon, PaintbrushIcon, PlugIcon } from '@modrinth/assets'
import { type Component, computed, onBeforeUnmount, ref, watch } from 'vue'

import BulletDivider from '#ui/components/base/BulletDivider.vue'
import { useFormatNumber } from '#ui/composables/format-number'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { formatProjectTypeSentence } from '#ui/utils/common-messages'

import type { ManagedContentSummaryItem, ManagedContentSummaryType } from '../../types'

const props = defineProps<{
	summary?: ManagedContentSummaryItem[]
	installing?: boolean
}>()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()

const messages = defineMessages({
	loading: {
		id: 'content.managed-card.summary.loading',
		defaultMessage: 'Loading managed content summary',
	},
	empty: {
		id: 'content.managed-card.summary.no-provided-content',
		defaultMessage: 'No provided content',
	},
	installing: {
		id: 'content.managed-card.summary.installing',
		defaultMessage: 'Installing content...',
	},
})

const icons: Record<ManagedContentSummaryType, Component> = {
	mod: BoxIcon,
	plugin: PlugIcon,
	datapack: BracesIcon,
	resourcepack: PaintbrushIcon,
	shader: GlassesIcon,
}

const MINIMUM_LOADING_TIME = 500
const minimumLoadingVisible = ref(false)
let minimumLoadingTimer: ReturnType<typeof setTimeout> | undefined

watch(
	() => props.summary === undefined,
	(isLoading) => {
		if (!isLoading) return

		minimumLoadingVisible.value = true
		clearTimeout(minimumLoadingTimer)
		minimumLoadingTimer = setTimeout(() => {
			minimumLoadingVisible.value = false
		}, MINIMUM_LOADING_TIME)
	},
	{ immediate: true },
)

onBeforeUnmount(() => clearTimeout(minimumLoadingTimer))

const showLoading = computed(() => props.summary === undefined || minimumLoadingVisible.value)
const visibleSummary = computed(() => props.summary?.filter((item) => item.count > 0))
</script>

<template>
	<div
		v-if="!installing && showLoading"
		class="flex h-6 items-center gap-2"
		role="status"
		:aria-label="formatMessage(messages.loading)"
	>
		<div class="h-5 w-20 animate-pulse rounded bg-surface-4" />
		<BulletDivider class="!mx-0" />
		<div class="h-5 w-32 animate-pulse rounded bg-surface-4" />
		<BulletDivider class="!mx-0" />
		<div class="h-5 w-20 animate-pulse rounded bg-surface-4" />
	</div>

	<span v-else-if="installing" class="font-medium leading-6 text-secondary" role="status">
		{{ formatMessage(messages.installing) }}
	</span>

	<div v-else-if="visibleSummary?.length" class="flex flex-wrap items-center gap-2 text-secondary">
		<template v-for="(item, index) in visibleSummary" :key="item.type">
			<BulletDivider v-if="index > 0" class="!mx-0" />
			<div class="flex items-center gap-1.5 whitespace-nowrap">
				<component :is="icons[item.type]" aria-hidden="true" class="size-5 shrink-0" />
				<span class="font-medium leading-6">
					{{ formatNumber(item.count) }}
					{{ formatProjectTypeSentence(formatMessage, item.type, item.count) }}
				</span>
			</div>
		</template>
	</div>

	<span v-else class="font-medium leading-6 text-secondary">
		{{ formatMessage(messages.empty) }}
	</span>
</template>
