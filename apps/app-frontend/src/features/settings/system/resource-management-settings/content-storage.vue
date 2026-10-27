<script setup lang="ts">
import { LoaderCircleIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	ProgressBar,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'

type StoreUsage = {
	unique_bytes: number
	shared_bytes: number
	unused_cache_bytes: number
	private_copy_bytes: number
	object_count: number
	cache_limit_bytes: number
}

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const gibibyte = 1024 ** 3
const queryClient = useQueryClient()
const storeUsageKey = ['content-store', 'usage'] as const
const {
	data: storeUsage,
	isPending: calculating,
	error: storeUsageError,
} = useQuery({
	queryKey: storeUsageKey,
	queryFn: async () => {
		const [usage] = await Promise.all([
			invoke<StoreUsage>('plugin:settings|store_usage'),
			new Promise<void>((resolve) => setTimeout(resolve, 1000)),
		])
		return usage
	},
})
const cacheLimitGiB = ref<number | undefined>()
watch(
	() => storeUsage.value?.cache_limit_bytes,
	(bytes) => {
		cacheLimitGiB.value = (bytes ?? 5 * gibibyte) / gibibyte
	},
	{ immediate: true },
)
watch(storeUsageError, (error) => {
	if (error) handleError(error)
})
const busy = computed(
	() => calculating.value || clearMutation.isPending.value || cacheLimitMutation.isPending.value,
)
const clearing = computed(() => clearMutation.isPending.value)
const clearedBytes = computed(() => clearMutation.data.value ?? null)
const totalBytes = computed(
	() => (storeUsage.value?.unique_bytes ?? 0) + (storeUsage.value?.private_copy_bytes ?? 0),
)

const messages = defineMessages({
	title: {
		id: 'app.settings.resource-management.store.title',
		defaultMessage: 'Content storage',
	},
	description: {
		id: 'app.settings.resource-management.store.description',
		defaultMessage: 'Save space by reusing game installs, mods and packs across your instances.',
	},
	calculating: {
		id: 'app.settings.resource-management.store.calculating',
		defaultMessage: 'Calculating',
	},
	stored: {
		id: 'app.settings.resource-management.store.total',
		defaultMessage: '{size} stored',
	},
	unique: {
		id: 'app.settings.resource-management.store.unique.label',
		defaultMessage: 'Unique',
	},
	shared: {
		id: 'app.settings.resource-management.store.shared.label',
		defaultMessage: 'Shared',
	},
	unused: {
		id: 'app.settings.resource-management.store.unused.label',
		defaultMessage: 'Orphaned',
	},
	clear: {
		id: 'app.settings.resource-management.store.clear',
		defaultMessage: 'Clear unused',
	},
	clearing: {
		id: 'app.settings.resource-management.store.clearing',
		defaultMessage: 'Clearing…',
	},
	clearDescription: {
		id: 'app.settings.resource-management.store.clear.description',
		defaultMessage: 'Delete unused downloaded content',
	},
	limit: {
		id: 'app.settings.resource-management.store.cache-limit.label',
		defaultMessage: 'Keep unused downloads',
	},
	limitDescription: {
		id: 'app.settings.resource-management.store.cache-limit.description',
		defaultMessage:
			'Keep up to this much unused content for future installs. Keeping more can save download time.',
	},
	limitUnit: {
		id: 'app.settings.resource-management.store.cache-limit.unit',
		defaultMessage: 'GiB',
	},
	cleared: {
		id: 'app.settings.resource-management.store.cleared',
		defaultMessage: 'Freed {size} of disk space.',
	},
	nothingToClear: {
		id: 'app.settings.resource-management.store.nothing-to-clear',
		defaultMessage: 'No unused content to clear.',
	},
})

const categories = computed(() => {
	const unused = storeUsage.value?.unused_cache_bytes ?? 0
	const shared = storeUsage.value?.shared_bytes ?? 0
	return [
		{
			id: 'unique',
			label: messages.unique,
			bytes: Math.max(0, totalBytes.value - unused - shared),
			color: 'bg-blue',
		},
		{
			id: 'shared',
			label: messages.shared,
			bytes: shared,
			color: 'bg-purple',
		},
		{
			id: 'unused',
			label: messages.unused,
			bytes: unused,
			color: 'bg-gray',
		},
	]
})

const clearMutation = useMutation({
	mutationFn: () => invoke<number>('plugin:settings|store_cleanup'),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: storeUsageKey }),
	onError: handleError,
})

function clearUnused() {
	if (busy.value) return
	clearMutation.reset()
	clearMutation.mutate()
}

const cacheLimitMutation = useMutation({
	mutationFn: (bytes: number) => invoke('plugin:settings|store_set_cache_limit', { bytes }),
	onSuccess: (_result, bytes) => {
		queryClient.setQueryData<StoreUsage>(storeUsageKey, (usage) =>
			usage ? { ...usage, cache_limit_bytes: bytes } : usage,
		)
	},
	onError: (error) => {
		cacheLimitGiB.value = (storeUsage.value?.cache_limit_bytes ?? 5 * gibibyte) / gibibyte
		handleError(error)
	},
})

function saveCacheLimit() {
	if (busy.value) return
	const bytes = Math.round(Number(cacheLimitGiB.value) * gibibyte)
	if (!Number.isSafeInteger(bytes) || bytes < 0) {
		cacheLimitGiB.value = (storeUsage.value?.cache_limit_bytes ?? 5 * gibibyte) / gibibyte
		return
	}
	cacheLimitMutation.mutate(bytes)
}
</script>

<template>
	<section
		v-if="calculating || storeUsage"
		class="@container flex flex-col gap-4"
		aria-labelledby="content-storage-title"
	>
		<div class="flex flex-col gap-1">
			<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
				<h2 id="content-storage-title" class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.title) }}
				</h2>
				<span class="font-semibold tabular-nums text-contrast" aria-live="polite">
					{{
						calculating
							? formatMessage(messages.calculating)
							: formatMessage(messages.stored, { size: formatBytes(totalBytes, 1) })
					}}
				</span>
			</div>
			<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		</div>

		<div class="flex flex-col gap-3">
			<ProgressBar
				v-if="calculating"
				:progress="0"
				waiting
				full-width
				class="[&>div:last-child]:h-3.5 [&>div:last-child]:bg-surface-4 [&>div:last-child>div]:bg-[--color-base]"
				aria-hidden="true"
			/>
			<div
				v-else
				aria-hidden="true"
				class="flex h-3.5 gap-0.5 overflow-hidden rounded-full bg-surface-4"
			>
				<template v-for="category in categories" :key="category.id">
					<div
						v-if="category.bytes > 0"
						class="h-full min-w-0 bg-gradient-to-b from-white/10 to-black/10 motion-safe:transition-[flex-grow] motion-safe:duration-300"
						:class="category.color"
						:style="{ flexGrow: category.bytes, flexBasis: '0%' }"
					/>
				</template>
			</div>
			<dl v-if="storeUsage" class="m-0 grid grid-cols-1 gap-3 @lg:grid-cols-3 @lg:gap-4">
				<div
					v-for="category in categories"
					:key="category.id"
					class="flex flex-wrap items-center gap-2"
				>
					<dt class="flex items-center gap-2">
						<span class="size-3 shrink-0 rounded-full" :class="category.color" aria-hidden="true" />
						{{ formatMessage(category.label) }}
					</dt>
					<dd class="m-0 ml-auto font-semibold tabular-nums text-contrast @lg:ml-0">
						{{ formatBytes(category.bytes, 1) }}
					</dd>
				</div>
			</dl>
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<Button
				v-tooltip="formatMessage(messages.clearDescription)"
				type="colored"
				color="red"
				:disabled="busy || !storeUsage || storeUsage.unused_cache_bytes === 0"
				:loading="clearing"
				@click="clearUnused"
			>
				<LoaderCircleIcon v-if="clearing" class="motion-safe:animate-spin" aria-hidden="true" />
				<TrashIcon v-else aria-hidden="true" />
				{{ formatMessage(clearing ? messages.clearing : messages.clear) }}
			</Button>
		</div>

		<div aria-live="polite" class="empty:hidden">
			<p v-if="clearedBytes !== null" class="m-0 text-sm text-green">
				{{
					clearedBytes > 0
						? formatMessage(messages.cleared, { size: formatBytes(clearedBytes, 1) })
						: formatMessage(messages.nothingToClear)
				}}
			</p>
		</div>

		<div class="flex flex-col gap-4 @lg:flex-row @lg:items-center">
			<div class="flex min-w-0 flex-1 flex-col gap-1">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					<label for="store-cache-limit">{{ formatMessage(messages.limit) }}</label>
				</h3>
				<p id="store-cache-limit-description" class="m-0">
					{{ formatMessage(messages.limitDescription) }}
				</p>
			</div>
			<Input
				id="store-cache-limit"
				v-model="cacheLimitGiB"
				type="number"
				:min="0"
				:step="0.5"
				aria-describedby="store-cache-limit-description"
				:disabled="busy"
				wrapper-class="w-full shrink-0 @lg:w-[42%]"
				@change="saveCacheLimit"
			>
				<template #trailing>
					<span class="text-sm text-secondary">{{ formatMessage(messages.limitUnit) }}</span>
				</template>
			</Input>
		</div>
		<hr class="m-0 w-full border-0 border-t border-solid border-surface-5" />
	</section>
</template>
