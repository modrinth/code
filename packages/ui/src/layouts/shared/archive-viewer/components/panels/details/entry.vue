<template>
	<section class="flex flex-col gap-3 p-3" :aria-labelledby="headingId">
		<div class="flex min-w-0 items-center gap-2">
			<component :is="icon" class="size-5 shrink-0" :class="iconClass" aria-hidden="true" />
			<div class="flex min-w-0 flex-1 flex-col">
				<span class="text-xs font-semibold uppercase tracking-wide text-secondary">{{ kind }}</span>
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
		</div>
		<DetailsList v-if="rows.length" :rows="rows" />
		<StatGrid v-if="sizes?.length" :stats="sizes" inline />
		<DetailsCounts v-if="counts" :counts="counts" />
		<StatGrid v-if="hashes?.length" :stats="hashes" grid />
	</section>
</template>

<script setup lang="ts">
import { CheckCircleIcon, CheckIcon, ClockIcon, LinkIcon, SpinnerIcon } from '@modrinth/assets'
import { type Component, computed, onBeforeUnmount, ref, useId } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import { useVIntl } from '#ui/composables/i18n'

import type { ArchiveDetailRow } from '../../../types'
import DetailsCounts from './counts.vue'
import DetailsList from './list.vue'
import { detailsMessages as messages } from './messages'
import StatGrid from './stat-grid.vue'
import type { ArchiveCounts, ArchiveReadStatus } from './types'

/**
 * One described item of the stack: what it is and its name (with how far it's been read, for an
 * archive), then its details, sizes, contents and checksums.
 */
const props = defineProps<{
	kind: string
	title: string
	icon: Component
	iconClass?: string
	/** Where the item sits in the archive, shown on hover and copied from a button. */
	path?: string
	status?: ArchiveReadStatus
	rows: ArchiveDetailRow[]
	sizes?: ArchiveDetailRow[]
	counts?: ArchiveCounts
	hashes?: ArchiveDetailRow[]
}>()

const { formatMessage } = useVIntl()
const headingId = useId()

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
