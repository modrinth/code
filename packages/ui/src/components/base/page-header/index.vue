<template>
	<div class="flex flex-col gap-2" :class="rootClass">
		<div class="flex flex-wrap items-start gap-4 max-md:flex-col" :class="rowClass">
			<div class="flex min-w-0 flex-1 gap-4" :class="mainClass">
				<PageHeaderLeading :items="leadingItems" />

				<div class="flex min-w-0 flex-col justify-center gap-2">
					<div class="flex flex-col justify-center gap-1.5">
						<div class="flex flex-wrap items-center gap-2">
							<h1
								class="m-0 min-w-0 max-w-full text-2xl font-semibold leading-none text-contrast"
								:class="titleClassValue"
							>
								{{ title }}
							</h1>
							<PageHeaderBadges :badges="badges" />
						</div>
						<p v-if="hasSummary" class="m-0 max-w-[44rem]" :class="summaryClass">
							<slot name="summary">{{ summary }}</slot>
						</p>
					</div>

					<PageHeaderMetadataList
						v-if="metadata.length"
						:items="metadata"
						wrapper-class="flex flex-wrap gap-3 max-md:hidden"
					/>
				</div>
			</div>

			<PageHeaderActions :actions="actions" />
		</div>

		<div v-if="metadata.length" class="flex justify-between md:hidden">
			<PageHeaderMetadataList :items="metadata" wrapper-class="flex flex-wrap gap-3" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed, useSlots } from 'vue'

import PageHeaderActions from './page-header-actions.vue'
import PageHeaderBadges from './page-header-badges.vue'
import PageHeaderLeading from './page-header-leading.vue'
import PageHeaderMetadataList from './page-header-metadata-list.vue'
import type { PageHeaderLeading as PageHeaderLeadingType, PageHeaderProps } from './types'

const props = withDefaults(defineProps<PageHeaderProps>(), {
	summary: null,
	leading: null,
	badges: () => [],
	metadata: () => [],
	actions: () => [],
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

const leadingItems = computed<PageHeaderLeadingType[]>(() => {
	if (!props.leading) return []
	return Array.isArray(props.leading) ? props.leading : [props.leading]
})
const rootClass = computed(() => [
	props.divider ? 'border-0 border-b border-solid border-divider' : '',
	props.bottomPadding ? 'pb-4' : '',
	props.headerClass,
])
const titleClassValue = computed(() => [props.truncateTitle ? 'truncate' : '', props.titleClass])
const summaryClass = computed(() => (props.disableLineClamp ? '' : 'line-clamp-2'))
const hasSummary = computed(() => !!props.summary || !!slots.summary)
</script>
