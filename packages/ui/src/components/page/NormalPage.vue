<script setup lang="ts">
import { injectPageContext } from '@modrinth/ui'

defineProps<{
	sidebar?: 'right' | 'left'
}>()
const { hierarchicalSidebarAvailable } = injectPageContext()
</script>
<template>
	<div
		class="ui-normal-page"
		:class="{
			'ui-normal-page--sidebar-left': sidebar === 'left' && !hierarchicalSidebarAvailable,
			'ui-normal-page--sidebar-right': sidebar === 'right' && !hierarchicalSidebarAvailable,
		}"
	>
		<div class="ui-normal-page__header">
			<slot name="header" />
		</div>
		<div class="ui-normal-page__content">
			<slot />
		</div>
		<template v-if="sidebar">
			<template v-if="hierarchicalSidebarAvailable">
				<Teleport to="#sidebar-teleport-target">
					<slot name="sidebar" />
				</Teleport>
			</template>
			<template v-else>
				<div class="ui-normal-page__sidebar">
					<slot name="sidebar" />
				</div>
			</template>
		</template>
	</div>
</template>
<style scoped>
.ui-normal-page {
	@apply grid gap-6 mx-auto py-4;
	width: min(calc(100% - 2rem), calc(80rem - 3rem));

	grid-template:
		'header'
		'content'
		'sidebar'
		/ 100%;
}

@media (width >= 64rem) {
	.ui-normal-page--sidebar-left {
		grid-template:
			'header header'
			'sidebar content'
			'sidebar dummy'
			/ 20rem 1fr;
	}

	.ui-normal-page--sidebar-right {
		grid-template:
			'header header'
			'content sidebar'
			'dummy sidebar'
			/ 1fr 20rem;
	}
}

.ui-normal-page__header {
	grid-area: header;
}

.ui-normal-page__content {
	grid-area: content;
}

.ui-normal-page__sidebar {
	grid-area: sidebar;
}
</style>
