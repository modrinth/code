<template>
	<TransferModal ref="transferModal" @success="refreshHistory" />
	<ConfirmModal
		ref="cancelModal"
		:title="`Cancel transfer batch #${cancellingBatchId}?`"
		description="This will cancel all transfers in this batch. This action cannot be undone."
		:proceed-icon="XCircleIcon"
		proceed-label="Cancel transfer"
		@proceed="confirmCancel"
	/>
	<div class="experimental-styles-within mx-auto max-w-[78.5rem] p-4">
		<div
			class="mb-6 flex items-end justify-between border-0 border-b border-solid border-divider pb-4"
		>
			<h1 class="m-0 text-2xl">Server transfers</h1>
			<ButtonStyled color="brand">
				<button @click="openTransferModal">
					<PlusIcon />
					New transfer
				</button>
			</ButtonStyled>
		</div>
		<div>
			<div v-if="loading" class="py-8 text-center text-secondary">Loading transfers...</div>
			<div v-else-if="error" class="py-8 text-center text-red">
				{{ error }}
			</div>
			<div v-else-if="!batches || batches.length === 0" class="py-8 text-center text-secondary">
				No transfer batches found.
			</div>
			<div v-else class="flex flex-col gap-3">
				<div
					v-for="batch in batches"
					:key="`batch-${batch.id}`"
					class="relative overflow-clip rounded-xl bg-bg-raised p-4"
				>
					<div class="absolute bottom-0 left-0 top-0 w-1" :class="getStatusColor(batch)" />
					<div class="ml-2 flex flex-col gap-2">
						<div class="flex items-center justify-between gap-4">
							<div class="flex items-center gap-3">
								<Avatar
									v-if="getUserById(batch.created_by)"
									:src="getUserById(batch.created_by)?.avatar_url"
									:alt="getUserById(batch.created_by)?.username"
									size="32px"
									circle
								/>
								<div v-else class="h-8 w-8 rounded-full bg-button-bg" />
								<div class="flex flex-col">
									<span class="font-semibold text-contrast"> Batch #{{ batch.id }} </span>
									<span class="text-sm text-secondary">
										by {{ getUserById(batch.created_by)?.username || batch.created_by }}
									</span>
								</div>
							</div>
							<div class="flex items-center gap-3">
								<div
									:style="{
										'--_color': getStatusStyle(batch).color,
										'--_bg-color': getStatusStyle(batch).bg,
									}"
								>
									<TagItem>
										{{ getStatusLabel(batch) }}
									</TagItem>
								</div>
								<span class="text-sm text-secondary">
									{{ batch.log_count }} transfer{{ batch.log_count === 1 ? '' : 's' }}
								</span>
								<ButtonStyled v-if="canCancel(batch)" color="red" color-fill="text">
									<button @click="showCancelModal(batch.id)">
										<XCircleIcon />
										Cancel
									</button>
								</ButtonStyled>
							</div>
						</div>
						<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-secondary">
							<span v-tooltip="dayjs(batch.created_at).format('MMMM D, YYYY [at] h:mm A')">
								Created {{ formatRelativeTime(batch.created_at) }}
							</span>
							<span>•</span>
							<span v-tooltip="dayjs(batch.scheduled_at).format('MMMM D, YYYY [at] h:mm A')">
								Scheduled {{ formatRelativeTime(batch.scheduled_at) }}
							</span>
							<template v-if="batch.provision_options?.region">
								<span>•</span>
								<span>Region: {{ batch.provision_options.region }}</span>
							</template>
							<template v-if="batch.provision_options?.node_tags?.length">
								<span>•</span>
								<span>Tags: {{ batch.provision_options.node_tags.join(', ') }}</span>
							</template>
						</div>
						<div v-if="batch.reason" class="text-sm">
							<span class="text-secondary">Reason:</span>
							<span class="ml-1 text-contrast">{{ truncateReason(batch.reason) }}</span>
						</div>
					</div>
				</div>
			</div>
			<!-- Pagination -->
			<div v-if="totalPages > 1" class="mt-6 flex justify-center">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon, XCircleIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ConfirmModal,
	injectNotificationManager,
	Pagination,
	TagItem,
	useRelativeTime,
} from '@modrinth/ui'
import type { User } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, ref } from 'vue'

import TransferModal from '~/components/ui/admin/TransferModal.vue'
import { useServersFetch } from '~/composables/servers/servers-fetch.ts'

const { addNotification } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()

// Types
interface ProvisionOptions {
	region?: string | null
	node_tags?: string[]
}

interface TransferBatch {
	id: number
	created_by: string
	created_at: string
	reason: string | null
	scheduled_at: string
	cancelled: boolean
	log_count: number
	provision_options: ProvisionOptions
}

interface HistoryResponse {
	batches: TransferBatch[]
	total: number
	page: number
	page_size: number
}

const transferModal = ref<InstanceType<typeof TransferModal>>()
const cancelModal = ref<InstanceType<typeof ConfirmModal>>()

const batches = ref<TransferBatch[]>([])
const total = ref(0)
const currentPage = ref(1)
const pageSize = 100
const loading = ref(true)
const error = ref<string | null>(null)

const users = ref<User[]>([])
const userMap = computed(() => new Map(users.value.map((u) => [u.id, u])))

const cancellingBatchId = ref<number | null>(null)

const totalPages = computed(() => Math.ceil(total.value / pageSize))

async function refreshHistory() {
	loading.value = true
	error.value = null
	try {
		const data = await useServersFetch<HistoryResponse>(
			`/transfers/history?page=${currentPage.value}&page_size=${pageSize}`,
			{ version: 'internal' },
		)
		batches.value = data.batches || []
		total.value = data.total || 0

		// Fetch users for avatars
		const userIds = [...new Set(batches.value.map((b) => b.created_by))]
		if (userIds.length > 0) {
			try {
				const fetchedUsers = (await useBaseFetch(`users?ids=${JSON.stringify(userIds)}`)) as User[]
				users.value = fetchedUsers
			} catch {
				// Silently fail - we'll just show user IDs instead
			}
		}
	} catch (err: any) {
		error.value = err?.data?.description ?? err?.message ?? String(err)
	} finally {
		loading.value = false
	}
}

function goToPage(page: number) {
	currentPage.value = page
	void refreshHistory()
}

await refreshHistory()

function getUserById(id: string): User | undefined {
	return userMap.value.get(id)
}

function getStatus(batch: TransferBatch): 'cancelled' | 'scheduled' | 'pending' {
	if (batch.cancelled) return 'cancelled'
	// Scheduled if less than 1 minute in the future (to account for processing)
	const scheduledTime = dayjs(batch.scheduled_at)
	const oneMinuteFromNow = dayjs().add(1, 'minute')
	if (scheduledTime.isBefore(oneMinuteFromNow)) return 'scheduled'
	return 'pending'
}

function getStatusLabel(batch: TransferBatch): string {
	const status = getStatus(batch)
	switch (status) {
		case 'cancelled':
			return 'Cancelled'
		case 'scheduled':
			return 'Scheduled'
		case 'pending':
			return 'Pending'
	}
}

function getStatusColor(batch: TransferBatch): string {
	const status = getStatus(batch)
	switch (status) {
		case 'cancelled':
			return 'bg-red'
		case 'scheduled':
			return 'bg-orange'
		case 'pending':
			return 'bg-blue'
	}
}

function getStatusStyle(batch: TransferBatch): { color: string; bg: string } {
	const status = getStatus(batch)
	switch (status) {
		case 'cancelled':
			return { color: 'var(--color-red)', bg: 'var(--color-red-bg)' }
		case 'scheduled':
			return { color: 'var(--color-orange)', bg: 'var(--color-orange-bg)' }
		case 'pending':
			return { color: 'var(--color-blue)', bg: 'var(--color-blue-bg)' }
	}
}

function canCancel(batch: TransferBatch): boolean {
	if (batch.cancelled) return false
	// can only cancel if more than 1 minute in the future
	const scheduledTime = dayjs(batch.scheduled_at)
	const oneMinuteFromNow = dayjs().add(1, 'minute')
	return scheduledTime.isAfter(oneMinuteFromNow)
}

function truncateReason(reason: string): string {
	if (reason.length <= 100) return reason
	return reason.slice(0, 100) + '...'
}

function openTransferModal(event?: Event) {
	transferModal.value?.show(event)
}

function showCancelModal(batchId: number) {
	cancellingBatchId.value = batchId
	cancelModal.value?.show()
}

async function confirmCancel() {
	if (!cancellingBatchId.value) return
	try {
		await useServersFetch('/transfers/cancel', {
			version: 'internal',
			method: 'POST',
			body: {
				batch_ids: [cancellingBatchId.value],
			},
		})
		addNotification({
			title: 'Transfer cancelled',
			text: `Batch #${cancellingBatchId.value} has been cancelled.`,
			type: 'success',
		})
		cancellingBatchId.value = null
		await refreshHistory()
	} catch (err: any) {
		addNotification({
			title: 'Error cancelling transfer',
			text: err?.data?.description ?? err?.message ?? String(err),
			type: 'error',
		})
	}
}
</script>
