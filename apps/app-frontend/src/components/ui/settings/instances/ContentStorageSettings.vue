<script setup lang="ts">
import { LoaderCircleIcon, ShieldCheckIcon, TrashIcon } from '@modrinth/assets'
import {
	Admonition,
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
import { computed, inject, ref, watch } from 'vue'

import {
	storeVerificationReport as report,
	verifyingStore,
	verifyStore,
} from '@/components/ui/download-manager/store-verification'
import { useAppEvent } from '@/composables/use-app-event'
import { get_all as getRunningProcesses } from '@/helpers/process'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

type StoreUsage = {
	unique_bytes: number
	shared_bytes: number
	unused_cache_bytes: number
	private_copy_bytes: number
	object_count: number
	cache_limit_bytes: number
}

const { handleError } = injectNotificationManager()
const settingsModal = inject(appSettingsModalContextKey, null)
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
const runningProcessesKey = ['processes', 'running'] as const
const { data: runningProcesses, error: runningProcessesError } = useQuery({
	queryKey: runningProcessesKey,
	queryFn: getRunningProcesses,
})
const hasRunningInstances = computed(() => (runningProcesses.value?.length ?? 0) > 0)
watch(runningProcessesError, (error) => {
	if (error) handleError(error)
})
useAppEvent('process', () => queryClient.invalidateQueries({ queryKey: runningProcessesKey }))
const busy = computed(
	() =>
		calculating.value ||
		actionMutation.isPending.value ||
		cacheLimitMutation.isPending.value ||
		verifyingStore.value,
)
const clearing = computed(
	() => actionMutation.isPending.value && actionMutation.variables.value === 'clear',
)
const clearedBytes = computed(() => actionMutation.data.value ?? null)
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
	empty: {
		id: 'app.settings.resource-management.store.empty',
		defaultMessage: 'Install mods or packs to see your storage usage here.',
	},
	clear: {
		id: 'app.settings.resource-management.store.clear',
		defaultMessage: 'Clear unused',
	},
	clearing: {
		id: 'app.settings.resource-management.store.clearing',
		defaultMessage: 'Clearing…',
	},
	repair: {
		id: 'app.settings.resource-management.store.repair',
		defaultMessage: 'Verify and repair',
	},
	repairing: {
		id: 'app.settings.resource-management.store.repairing',
		defaultMessage: 'Verifying and repairing…',
	},
	repairDescription: {
		id: 'app.settings.resource-management.store.repair.verify-description',
		defaultMessage: 'Check for missing or damaged files and repair them.',
	},
	repairRunningDescription: {
		id: 'app.settings.resource-management.store.repair.running-description',
		defaultMessage: 'This will close any running instances.',
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
	verified: {
		id: 'app.settings.resource-management.store.verified',
		defaultMessage: 'Verification complete',
	},
	attention: {
		id: 'app.settings.resource-management.store.attention',
		defaultMessage: 'Some files still need attention',
	},
	result: {
		id: 'app.settings.resource-management.store.verification-result',
		defaultMessage:
			'Checked {checked, plural, one {# file} other {# files}} and made {repaired, plural, one {# repair} other {# repairs}}.',
	},
	issues: {
		id: 'app.settings.resource-management.store.issues',
		defaultMessage: 'View {count, plural, one {# issue} other {# issues}}',
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

const actionMutation = useMutation({
	mutationFn: async (action: 'clear' | 'repair') => {
		if (action === 'repair') {
			const verification = verifyStore()
			settingsModal?.close()
			await verification
			return null
		}
		return invoke<number>('plugin:settings|store_cleanup')
	},
	onMutate: () => {
		report.value = null
	},
	onSuccess: () => queryClient.invalidateQueries({ queryKey: storeUsageKey }),
	onError: handleError,
})

function runAction(action: 'clear' | 'repair') {
	if (busy.value) return
	actionMutation.reset()
	actionMutation.mutate(action)
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
			<p v-if="storeUsage && totalBytes === 0" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.empty) }}
			</p>
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<Button
				v-tooltip="
					formatMessage(
						hasRunningInstances ? messages.repairRunningDescription : messages.repairDescription,
					)
				"
				:type="hasRunningInstances ? 'colored' : 'base'"
				:color="hasRunningInstances ? 'orange' : undefined"
				:disabled="busy || !storeUsage || storeUsage.object_count === 0"
				:loading="verifyingStore"
				@click="runAction('repair')"
			>
				<LoaderCircleIcon
					v-if="verifyingStore"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<ShieldCheckIcon v-else aria-hidden="true" />
				{{ formatMessage(verifyingStore ? messages.repairing : messages.repair) }}
			</Button>
			<Button
				v-tooltip="formatMessage(messages.clearDescription)"
				type="colored"
				color="red"
				:disabled="busy || !storeUsage || storeUsage.unused_cache_bytes === 0"
				:loading="clearing"
				@click="runAction('clear')"
			>
				<LoaderCircleIcon v-if="clearing" class="motion-safe:animate-spin" aria-hidden="true" />
				<TrashIcon v-else aria-hidden="true" />
				{{ formatMessage(clearing ? messages.clearing : messages.clear) }}
			</Button>
		</div>

		<div aria-live="polite" class="empty:hidden">
			<Admonition
				v-if="report"
				:type="report.issues.length ? 'warning' : 'success'"
				:header="formatMessage(report.issues.length ? messages.attention : messages.verified)"
			>
				<p class="m-0">
					{{
						formatMessage(messages.result, { checked: report.checked, repaired: report.repaired })
					}}
				</p>
				<details v-if="report.issues.length" class="mt-2">
					<summary class="cursor-pointer font-semibold">
						{{ formatMessage(messages.issues, { count: report.issues.length }) }}
					</summary>
					<ul class="mb-0 flex flex-col gap-2 pl-5">
						<li
							v-for="(issue, index) in report.issues"
							:key="`${issue.sha512}-${index}`"
							class="break-words"
						>
							{{ issue.message }}
						</li>
					</ul>
				</details>
			</Admonition>
			<p v-else-if="clearedBytes !== null" class="m-0 text-sm text-green">
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
