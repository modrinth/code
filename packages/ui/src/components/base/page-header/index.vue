<template>
	<div class="flex flex-col gap-2" :class="rootClass">
		<div class="flex flex-wrap items-start gap-4 max-md:flex-col" :class="rowClass">
			<div class="flex min-w-0 flex-1 gap-4" :class="mainClass">
				<div v-if="$slots.leading" class="flex shrink-0 items-center gap-4">
					<slot name="leading" />
				</div>

				<div class="flex min-w-0 flex-col justify-center gap-2">
					<div class="flex flex-col justify-center gap-1.5">
						<div class="flex flex-wrap items-center gap-2">
							<h1
								class="m-0 min-w-0 max-w-full text-2xl font-semibold leading-none text-contrast"
								:class="titleClassValue"
							>
								{{ title }}
							</h1>
							<slot name="badges" />
						</div>
						<p v-if="hasSummary" class="m-0 max-w-[44rem]" :class="summaryClass">
							<slot name="summary">{{ summary }}</slot>
						</p>
					</div>

					<div v-if="$slots.metadata" class="max-md:hidden">
						<slot name="metadata" />
					</div>
				</div>
			</div>

			<slot name="actions" />
		</div>

		<div v-if="$slots.metadata" class="flex justify-between md:hidden">
			<slot name="metadata" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, useSlots } from 'vue'

import type { PageHeaderProps } from './types'

const props = withDefaults(defineProps<PageHeaderProps>(), {
	summary: null,
	headerClass: '',
	rowClass: '',
	mainClass: '',
	titleClass: '',
	truncateTitle: false,
	divider: true,
	bottomPadding: true,
	disableLineClamp: false,
})

const slots = useSlots()

const rootClass = computed(() => [
	props.divider ? 'border-0 border-b border-solid border-divider' : '',
	props.bottomPadding ? 'pb-4' : '',
	props.headerClass,
])
const titleClassValue = computed(() => [props.truncateTitle ? 'truncate' : '', props.titleClass])
const summaryClass = computed(() => (props.disableLineClamp ? '' : 'line-clamp-2'))
const hasSummary = computed(() => !!props.summary || !!slots.summary)
</script>
