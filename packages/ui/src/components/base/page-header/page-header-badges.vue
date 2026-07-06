<template>
	<template v-for="badge in badges" :key="badge.id">
		<component
			:is="badge.component"
			v-if="badge.type === 'component'"
			:class="badge.class"
			v-bind="badge.componentProps"
		/>
		<PageHeaderInteractiveWrapper
			v-else
			:to="badge.to"
			:clickable="!!badge.onClick"
			:disabled="badge.disabled"
			:tooltip="badge.tooltip"
			:aria-label="badgeLabel(badge)"
			:base-class="badgeClass(badge)"
			:interactive-class="interactiveBadgeClass"
			:style="badge.style"
			@click="(event) => badge.onClick?.(event)"
		>
			<PageHeaderBadgeContent :badge="badge" />
		</PageHeaderInteractiveWrapper>
	</template>
</template>

<script setup lang="ts">
import PageHeaderBadgeContent from './page-header-badge-content.vue'
import PageHeaderInteractiveWrapper from './page-header-interactive-wrapper.vue'
import type { PageHeaderBadge, PageHeaderBadgeContent as PageHeaderBadgeContentType } from './types'

defineProps<{
	badges: PageHeaderBadge[]
}>()

const baseBadgeClass =
	'inline-flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-button-bg px-2 py-1 text-sm font-semibold leading-none text-secondary text-nowrap [&>svg]:size-4 [&>svg]:shrink-0'
const interactiveBadgeClass = 'm-0 cursor-pointer hover:underline'

function badgeClass(badge: PageHeaderBadgeContentType) {
	return [baseBadgeClass, badge.class]
}

function badgeLabel(badge: PageHeaderBadgeContentType) {
	return badge.ariaLabel ?? badge.label ?? badge.tooltip
}
</script>
