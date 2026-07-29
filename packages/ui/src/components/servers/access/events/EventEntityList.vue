<template>
	<span
		class="inline-flex max-w-full min-w-0 items-center gap-x-1 align-middle"
		:class="
			singleLine
				? 'flex-wrap gap-y-0.5 whitespace-normal @[800px]:flex-nowrap @[800px]:overflow-hidden @[800px]:whitespace-nowrap'
				: 'flex-wrap gap-y-0.5'
		"
	>
		<template v-for="(entity, index) in visibleEntities" :key="entity.id">
			<EventEntityLink
				:entity="entity"
				:text-weight="entityTextWeight"
				:class="singleLine ? 'min-w-0 shrink' : ''"
			/>
			<span v-if="index < visibleEntities.length - 1" class="shrink-0 text-secondary">,</span>
		</template>
		<Tooltip
			v-if="hiddenCount > 0"
			theme="dismissable-prompt"
			class="inline-flex shrink-0 items-center"
			:triggers="['hover', 'focus']"
			:popper-triggers="['hover', 'focus']"
			popper-class="v-popper--interactive audit-log-entity-list-popper"
			placement="top"
			:delay="{ show: 200, hide: 100 }"
			no-auto-focus
		>
			<button
				type="button"
				class="inline-flex min-w-0 cursor-help items-center rounded-full border border-solid border-surface-5 bg-surface-4 px-1.5 py-1 leading-none text-xs text-secondary"
				:aria-label="hiddenTooltip"
			>
				{{ formatMessage(messages.hiddenCount, { count: hiddenCount }) }}
			</button>
			<template #popper>
				<div class="relative max-w-[22rem]">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-3"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-3"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-3 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>
					<div
						ref="hiddenEntitiesScrollContainer"
						class="flex flex-col gap-2 overflow-y-auto overscroll-contain py-0.5"
						:style="{ maxHeight: hiddenEntitiesMaxHeight }"
						@scroll="checkScrollState"
					>
						<EventEntityLink
							v-for="entity in hiddenEntities"
							:key="entity.id"
							:entity="entity"
							:text-weight="entityTextWeight"
							class="min-w-0 pr-4"
							stack-secondary
						/>
					</div>
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-3"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-3"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-3 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>
			</template>
		</Tooltip>
	</span>
</template>

<script setup lang="ts">
import { Tooltip } from 'floating-vue'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../../composables/i18n'
import { useScrollIndicator } from '../../../../composables/scroll-indicator'
import EventEntityLink from './EventEntityLink.vue'
import type { EventEntity } from './types'

const props = withDefaults(
	defineProps<{
		entities: EventEntity[]
		limit?: number
		singleLine?: boolean
		entityTextWeight?: 'medium' | 'semibold'
	}>(),
	{
		limit: 3,
		singleLine: true,
		entityTextWeight: 'medium',
	},
)

const { formatMessage, locale } = useVIntl()
const hiddenEntitiesScrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(
	hiddenEntitiesScrollContainer,
)

const TOOLTIP_VISIBLE_ROWS = 8
const TOOLTIP_ENTITY_HEIGHT_REM = 1.75
const TOOLTIP_ENTITY_GAP_REM = 0.5
const hiddenEntitiesMaxHeight = `${
	TOOLTIP_VISIBLE_ROWS * TOOLTIP_ENTITY_HEIGHT_REM +
	(TOOLTIP_VISIBLE_ROWS - 1) * TOOLTIP_ENTITY_GAP_REM
}rem`

const messages = defineMessages({
	hiddenCount: {
		id: 'servers.audit-log.event.entity-list.hidden-count',
		defaultMessage: '+{count, number}',
	},
})

const visibleEntities = computed(() => props.entities.slice(0, props.limit))
const hiddenEntities = computed(() => props.entities.slice(props.limit))
const hiddenCount = computed(() => hiddenEntities.value.length)
const hiddenTooltip = computed(() => {
	void locale.value
	return new Intl.ListFormat(locale.value, {
		style: 'long',
		type: 'conjunction',
	}).format(hiddenEntities.value.map((entity) => entity.label))
})
</script>

<style lang="scss">
.v-popper__popper.v-popper--theme-dismissable-prompt.audit-log-entity-list-popper {
	.v-popper__inner {
		padding-right: 0 !important;
	}
}
</style>
