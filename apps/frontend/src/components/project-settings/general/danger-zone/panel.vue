<template>
	<div class="danger-zone-panel rounded-2xl border border-solid bg-surface-2 px-6">
		<slot />
	</div>
</template>

<style scoped>
.danger-zone-panel {
	border-color: color-mix(in srgb, var(--color-red) 30%, transparent);
}

:deep(.danger-zone-row) {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 16px;
	padding: 24px 0;
}
:deep(.danger-zone-row + .danger-zone-row) {
	border-top: 1px solid var(--surface-4);
}
@media (max-width: 640px) {
	:deep(.danger-zone-row) {
		align-items: flex-start;
		flex-direction: column;
	}
}
</style>
