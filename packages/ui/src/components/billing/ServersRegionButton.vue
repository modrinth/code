<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { regionOverrides } from '../../utils/regions'
import ServersRegionPing from './ServersRegionPing.vue'
const { formatMessage } = useVIntl()

const currentRegion = defineModel<string | undefined>({ required: true })

const props = defineProps<{
	region: Archon.Servers.v1.Region
	ping?: number
	bestPing?: boolean
	outOfStock?: boolean
}>()

const messages = defineMessages({
	outOfStock: {
		id: 'servers.region.out-of-stock',
		defaultMessage: '(Out of stock)',
	},
})

const isCurrentRegion = computed(() => currentRegion.value === props.region.shortcode)
const flag = computed(
	() =>
		regionOverrides[props.region.shortcode]?.flag ??
		`https://flagcdn.com/${props.region.country_code}.svg`,
)
const overrideTitle = computed(() => regionOverrides[props.region.shortcode]?.name)
const title = computed(() =>
	overrideTitle.value ? formatMessage(overrideTitle.value) : props.region.display_name,
)
const locationSubtitle = computed(() =>
	overrideTitle.value ? props.region.display_name : undefined,
)

function setRegion() {
	currentRegion.value = props.region.shortcode
}
</script>

<template>
	<button
		:disabled="outOfStock"
		class="rounded-2xl p-4 font-semibold transition-all border-2 border-solid flex flex-col items-center justify-center gap-3"
		:class="{
			'bg-button-bg border-transparent text-primary': !isCurrentRegion,
			'bg-brand-highlight border-brand text-contrast': isCurrentRegion,
			'opacity-50 cursor-not-allowed': outOfStock,
			'hover:text-contrast active:scale-95 hover:brightness-[--hover-brightness] focus-visible:brightness-[--hover-brightness] ':
				!outOfStock,
		}"
		@click="setRegion"
	>
		<img
			v-if="flag"
			class="aspect-[16/10] max-w-16 w-full object-cover rounded-md border-1 border-solid"
			:class="[
				isCurrentRegion ? 'border-brand' : 'border-button-border',
				{ 'saturate-[0.25]': outOfStock },
			]"
			:src="flag"
			alt=""
			aria-hidden="true"
		/>
		<span class="flex flex-col gap-1 items-center">
			<span>{{ title }}</span>
			<span v-if="outOfStock" class="text-sm text-secondary">
				{{ formatMessage(messages.outOfStock) }}
			</span>
			<span
				class="text-xs flex items-center flex-wrap justify-center gap-1 text-secondary font-medium"
			>
				<template v-if="locationSubtitle">
					<span>
						{{ locationSubtitle }}
					</span>
					<span>•</span>
				</template>
				<ServersRegionPing :ping="ping" />
			</span>
		</span>
	</button>
</template>
