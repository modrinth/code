<script setup lang="ts">
import { SignalIcon, SpinnerIcon } from '@modrinth/assets'
import { getPingLevel } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

import type { ServerRegion } from '../../utils/billing'
import { regionOverrides } from '../../utils/regions'

const { formatMessage } = useVIntl()

const currentRegion = defineModel<string | undefined>({ required: true })

const props = defineProps<{
	region: ServerRegion
	ping?: number
	bestPing?: boolean
	outOfStock?: boolean
}>()

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
const pingLevel = computed(() => getPingLevel(props.ping ?? 0))

function setRegion() {
	currentRegion.value = props.region.shortcode
}
</script>

<template>
	<button
		:disabled="outOfStock"
		class="rounded-2xl p-4 font-semibold transition-all border-2 border-solid flex flex-col items-center gap-3"
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
			<span class="flex items-center gap-1 flex-wrap justify-center">
				{{ title }}
				<span v-if="outOfStock" class="text-sm text-secondary">(Out of stock)</span>
			</span>
			<span class="text-xs flex items-center gap-1 text-secondary font-medium">
				<template v-if="locationSubtitle">
					<span>
						{{ locationSubtitle }}
					</span>
					<span v-if="ping !== -1">•</span>
				</template>
				<template v-if="ping !== -1">
					<SignalIcon
						v-if="ping"
						aria-hidden="true"
						:style="`--_signal-${pingLevel}: ${pingLevel <= 2 ? 'var(--color-red)' : pingLevel <= 4 ? 'var(--color-orange)' : 'var(--color-green)'}`"
						stroke-width="3px"
						class="shrink-0"
					/>
					<SpinnerIcon v-else class="animate-spin" />
					<template v-if="ping"> {{ ping }}ms </template>
					<span v-else> Testing connection... </span>
				</template>
			</span>
		</span>
	</button>
</template>
