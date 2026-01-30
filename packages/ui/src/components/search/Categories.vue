<template>
	<div class="categories">
		<slot />
		<span v-for="category in categories.filter((x) => !!x)" :key="category">
			<component :is="getTagIcon(category)" v-if="getTagIcon(category)" />
			{{ formatTag(formatMessage, category) }}
		</span>
	</div>
</template>
<script setup lang="ts">
import { getTagIcon } from '@modrinth/assets'

import { useVIntl } from '../../composables'
import { formatTag } from '../../utils/tag-messages.ts'

const { formatMessage } = useVIntl()

defineProps<{
	categories: string[]
}>()
</script>

<style lang="scss" scoped>
.categories {
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	gap: var(--gap-sm);

	:deep(span) {
		display: flex;
		flex-direction: row;
		align-items: center;

		&:not(.version-badge) {
			color: var(--color-gray);
		}

		svg {
			width: 1rem;
			margin-right: 0.2rem;
		}
	}
}
</style>
