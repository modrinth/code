<template>
	<div class="page-header-metadata flex min-w-0 flex-wrap items-center gap-x-2 gap-y-2">
		<slot />
	</div>
</template>

<style scoped>
.page-header-metadata
	:deep([data-page-header-metadata-item]:first-child .page-header-metadata-item-divider) {
	display: none;
}
</style>
