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
			<DetailsList v-if="rows?.length" :rows="rows" />
			<StatGrid v-if="sizes?.length" :stats="sizes" inline />
			<DetailsCounts v-if="counts && countsVisibility === 'always'" :counts="counts" />
		</div>
		<template v-if="hasDetails">
			<div v-if="open" :id="detailsId" class="flex flex-col gap-3 px-3 pb-3">
				<slot name="details" />
				<DetailsList v-if="more?.length" :rows="more" />
				<DetailsCounts v-if="counts && countsVisibility === 'details'" :counts="counts" />
				<HashList v-if="hash && shownHashTypes.length" :resolve="hash" :types="shownHashTypes" />
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
import type { HashType } from '#ui/helpers/toolbox/file'

import type { ArchiveDetailRow } from '../../types'
import type { ArchiveCounts, ArchiveReadStatus } from '../../types'
import {
	DEFAULT_ARCHIVE_HASH_TYPES,
	injectArchiveDetailsSettings,
} from '../../providers/details-settings'
import DetailsCounts from './counts.vue'
import DetailsToggle from './details-toggle.vue'
import HashList from './hash-list.vue'
import DetailsList from './list.vue'
import { sectionMessages as messages } from './messages'
import StatGrid from './stat-grid.vue'

/**
 * One described item of the details stack: what it is and its name (with how far it's been read,
 * for an archive), then its details and sizes. In-depth details and hashes are tucked behind a
 * pull tab along its bottom, open or closed for every item sharing its `detailsKey`; contents
 * counts show as the viewer's settings say. Hosts can use it for items of their own (e.g. the
 * project an archive was published by), with an `icon` slot in place of the icon, `actions`
 * beside the name, their own content (the default slot) before the details and more of it
 * (the `details` slot) behind the pull tab.
 */
const props = defineProps<{
	kind: string
	title: string
	icon?: Component
	iconClass?: string
	/** Where the item sits in the archive, shown on hover and copied from a button. */
	path?: string
	status?: ArchiveReadStatus
	rows?: ArchiveDetailRow[]
	/** In-depth details, folded away by default. */
	more?: ArchiveDetailRow[]
	sizes?: ArchiveDetailRow[]
	counts?: ArchiveCounts
	/** Computes the item's hash of a type, for those the viewer's settings list. */
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
const settings = injectArchiveDetailsSettings(null)

const open = useLocalStorage(`archive-viewer-details:open:${props.detailsKey}`, false, {
	initOnMounted: true,
})

const countsVisibility = computed(() => settings?.contentCounts.value ?? 'details')

const shownHashTypes = computed(() => {
	const selected = settings?.hashTypes.value ?? DEFAULT_ARCHIVE_HASH_TYPES
	return props.hashTypes ? selected.filter((type) => props.hashTypes!.includes(type)) : selected
})

const hasDetails = computed(
	() =>
		!!slots.details ||
		!!props.more?.length ||
		(!!props.counts && countsVisibility.value === 'details') ||
		(!!props.hash && shownHashTypes.value.length > 0),
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
