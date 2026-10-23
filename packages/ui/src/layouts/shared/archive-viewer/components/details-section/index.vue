<template>
	<section class="flex flex-col" :aria-labelledby="headingId">
		<div class="flex flex-col gap-3 p-3">
			<div class="flex min-w-0 items-center gap-2">
				<slot name="icon">
					<component
						:is="icon"
						v-if="icon"
						class="size-5 shrink-0"
						:class="iconClass"
						aria-hidden="true"
					/>
				</slot>
				<div class="flex min-w-0 flex-1 flex-col">
					<span class="text-xs font-semibold uppercase tracking-wide text-secondary">{{
						kind
					}}</span>
					<div class="flex min-w-0 items-center gap-1.5">
						<h3
							:id="headingId"
							class="m-0 truncate text-base font-semibold text-contrast"
							:title="title"
						>
							{{ title }}
						</h3>
						<component
							:is="statusIcon.icon"
							v-if="statusIcon"
							v-tooltip="statusIcon.label"
							class="size-4 shrink-0"
							:class="statusIcon.class"
							role="img"
							:aria-label="statusIcon.label"
						/>
					</div>
				</div>
				<IconButton
					v-if="path"
					v-tooltip="path"
					type="quiet"
					size="xs"
					class="shrink-0"
					:label="formatMessage(copied ? messages.pathCopied : messages.copyPath)"
					@click="copyPath"
				>
					<CheckIcon v-if="copied" class="text-green" aria-hidden="true" />
					<LinkIcon v-else aria-hidden="true" />
				</IconButton>
				<slot name="actions" />
			</div>
			<slot />
			<DetailsList v-if="shownRows.always.length" :rows="shownRows.always" />
			<StatGrid v-if="shownSizes.always.length" :stats="shownSizes.always" inline />
			<DetailsCounts v-if="counts && countsVisibility === 'always'" :counts="counts" />
			<HashList
				v-if="hash && hashTypesShown.always.length"
				:resolve="hash"
				:types="hashTypesShown.always"
			/>
		</div>
		<template v-if="hasDetails">
			<div v-if="open" :id="detailsId" class="flex flex-col gap-3 px-3 pb-3">
				<slot name="details" />
				<DetailsList v-if="detailRows.length" :rows="detailRows" />
				<StatGrid v-if="shownSizes.details.length" :stats="shownSizes.details" inline />
				<DetailsCounts v-if="counts && countsVisibility === 'details'" :counts="counts" />
				<HashList
					v-if="hash && hashTypesShown.details.length"
					:resolve="hash"
					:types="hashTypesShown.details"
				/>
			</div>
			<DetailsToggle v-model:open="open" :controls="detailsId" />
		</template>
	</section>
</template>

<script setup lang="ts">
import { CheckCircleIcon, CheckIcon, ClockIcon, LinkIcon, SpinnerIcon } from '@modrinth/assets'
import { useLocalStorage } from '@vueuse/core'
import { type Component, computed, onBeforeUnmount, ref, useId, type VNode } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import { useVIntl } from '#ui/composables/i18n'
import { HASH_TYPES, type HashType } from '#ui/helpers/toolbox/file'

import { type DetailVisibility, useArchiveDetailSettings } from '../../settings'
import type { ArchiveCounts, ArchiveDetailRow, ArchiveReadStatus } from '../../types'
import DetailsCounts from './counts.vue'
import DetailsToggle from './details-toggle.vue'
import HashList from './hash-list.vue'
import DetailsList from './list.vue'
import { sectionMessages as messages } from './messages'
import StatGrid from './stat-grid.vue'

/**
 * One described item of the details stack: what it is and its name (with how far it's been read,
 * for an archive), then its details. Each row, size, the contents counts and each hash shows
 * always, only behind the pull tab along the section's bottom, or never, as the row (or the
 * viewer's settings) say; the pull tab is open or closed for every item sharing its
 * `detailsKey`. Hosts can use it for items of their own (e.g. the project an archive was
 * published by), with an `icon` slot in place of the icon, `actions` beside the name, their own
 * content (the default slot) before the details and more of it (the `details` slot) behind the
 * pull tab.
 */
const props = defineProps<{
	kind: string
	title: string
	icon?: Component
	iconClass?: string
	/** Where the item sits in the archive, shown on hover and copied from a button. */
	path?: string
	status?: ArchiveReadStatus
	/** Details shown as their `visibility` says, always by default. */
	rows?: ArchiveDetailRow[]
	/** In-depth details, only shown behind the pull tab. */
	more?: ArchiveDetailRow[]
	/** Sizes, shown as their `visibility` says, always by default. */
	sizes?: ArchiveDetailRow[]
	counts?: ArchiveCounts
	/** Computes the item's hash of a type, for those the viewer's settings show. */
	hash?: (type: HashType) => Promise<string>
	/** Hash types the item can have, when it can't have every kind. */
	hashTypes?: HashType[]
	/** Items sharing a key show or hide their details together, e.g. every nested archive. */
	detailsKey: string
}>()

const slots = defineSlots<{
	icon?: () => VNode[]
	actions?: () => VNode[]
	default?: () => VNode[]
	details?: () => VNode[]
}>()

const { formatMessage } = useVIntl()
const headingId = useId()
const detailsId = useId()
const settings = useArchiveDetailSettings()

const open = useLocalStorage(`archive-viewer-details:open:${props.detailsKey}`, false, {
	initOnMounted: true,
})

const countsVisibility = computed(() => settings.visibility('contentCounts'))

/** Rows split by where they show; ones never shown are left out. */
function byVisibility(rows: ArchiveDetailRow[] | undefined) {
	const split: Record<Exclude<DetailVisibility, 'never'>, ArchiveDetailRow[]> = {
		always: [],
		details: [],
	}
	for (const row of rows ?? []) {
		const visibility = row.visibility ?? 'always'
		if (visibility !== 'never') split[visibility].push(row)
	}
	return split
}

const shownRows = computed(() => byVisibility(props.rows))
const shownSizes = computed(() => byVisibility(props.sizes))
const detailRows = computed(() => [...shownRows.value.details, ...(props.more ?? [])])

/** The item's hash types split by where the settings show them, in the order they're listed. */
const hashTypesShown = computed(() => {
	const split: Record<Exclude<DetailVisibility, 'never'>, HashType[]> = { always: [], details: [] }
	for (const type of HASH_TYPES) {
		if (props.hashTypes && !props.hashTypes.includes(type)) continue
		const visibility = settings.hashVisibility(type)
		if (visibility !== 'never') split[visibility].push(type)
	}
	return split
})

const hasDetails = computed(
	() =>
		!!slots.details ||
		detailRows.value.length > 0 ||
		shownSizes.value.details.length > 0 ||
		(!!props.counts && countsVisibility.value === 'details') ||
		(!!props.hash && hashTypesShown.value.details.length > 0),
)

const copied = ref(false)
let copiedReset: ReturnType<typeof setTimeout> | undefined

async function copyPath() {
	if (!props.path) return
	await navigator.clipboard.writeText(props.path)
	copied.value = true
	clearTimeout(copiedReset)
	copiedReset = setTimeout(() => (copied.value = false), 2000)
}

onBeforeUnmount(() => clearTimeout(copiedReset))

const statusIcon = computed(() => {
	switch (props.status) {
		case 'read':
			return {
				icon: CheckCircleIcon,
				class: 'text-green',
				label: formatMessage(messages.statusRead),
			}
		case 'reading':
			return {
				icon: SpinnerIcon,
				class: 'animate-spin text-secondary',
				label: formatMessage(messages.statusReading),
			}
		case 'pending':
			return {
				icon: ClockIcon,
				class: 'text-secondary',
				label: formatMessage(messages.statusPending),
			}
		default:
			return null
	}
})
</script>
