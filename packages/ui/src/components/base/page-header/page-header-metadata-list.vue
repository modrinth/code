<template>
	<div :class="wrapperClass">
		<div class="flex min-w-0 flex-wrap items-center gap-2">
			<template v-for="(item, index) in items" :key="item.id">
				<BulletDivider v-if="index > 0" class="shrink-0" />
				<component
					:is="item.component"
					v-if="item.type === 'component'"
					:class="item.class"
					v-bind="item.componentProps"
				/>
				<PageHeaderInteractiveWrapper
					v-else
					:to="item.to"
					:clickable="!!item.onClick"
					:disabled="item.disabled"
					:tooltip="item.tooltip"
					:aria-label="metadataLabel(item)"
					:base-class="metadataClass(item)"
					:interactive-class="interactiveMetadataClass"
					@click="(event) => item.onClick?.(event)"
				>
					<PageHeaderMetadataItemContent :item="item" />
				</PageHeaderInteractiveWrapper>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import BulletDivider from '../BulletDivider.vue'
import PageHeaderInteractiveWrapper from './page-header-interactive-wrapper.vue'
import PageHeaderMetadataItemContent from './page-header-metadata-item-content.vue'
import type {
	PageHeaderClass,
	PageHeaderMetadataContentItem,
	PageHeaderMetadataItem,
} from './types'

defineProps<{
	items: PageHeaderMetadataItem[]
	wrapperClass?: PageHeaderClass
}>()

const baseMetadataClass = 'flex min-w-0 items-center gap-2 font-medium text-secondary text-nowrap'
const interactiveMetadataClass = 'm-0 cursor-pointer border-0 bg-transparent p-0 hover:underline'

function metadataClass(item: PageHeaderMetadataContentItem) {
	return [baseMetadataClass, item.class]
}

function metadataLabel(item: PageHeaderMetadataContentItem) {
	return item.ariaLabel ?? item.label ?? undefined
}
</script>
