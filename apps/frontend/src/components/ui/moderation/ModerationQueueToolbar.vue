<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col justify-between gap-2 lg:flex-row">
			<Input
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="formatMessage(commonMessages.searchPlaceholder)"
				clearable
				size="medium"
				wrapper-class="min-w-0 flex-1"
				@input="$emit('search')"
			/>
			<div
				class="flex flex-col items-stretch justify-end gap-2 sm:flex-row sm:items-center lg:flex-shrink-0"
			>
				<slot name="actions" />
			</div>
		</div>
		<div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
			<div class="flex min-w-0 flex-wrap items-center gap-3">
				<div v-if="loading" class="h-6 w-64 animate-pulse rounded-full bg-surface-3"></div>
				<slot v-else name="meta" />
			</div>
			<div class="flex shrink-0 items-center justify-end gap-2 sm:ml-auto">
				<div v-if="loading && !(totalPages > 1)" class="flex items-center gap-2">
					<div
						v-for="i in 3"
						:key="`loading-skeleton-${i}`"
						class="h-9 w-9 animate-pulse rounded-full bg-surface-3"
					></div>
				</div>
				<template v-else>
					<slot name="pagination-extra" />
					<Pagination
						v-if="totalPages > 1"
						:page="page"
						:count="totalPages"
						@switch-page="$emit('switch-page', $event)"
					/>
				</template>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { commonMessages, Input, Pagination, useVIntl } from '@modrinth/ui'

const query = defineModel<string>({ required: true })

defineProps<{
	page: number
	totalPages: number
	loading?: boolean
}>()

defineEmits<{
	search: []
	'switch-page': [page: number]
}>()

const { formatMessage } = useVIntl()
</script>
