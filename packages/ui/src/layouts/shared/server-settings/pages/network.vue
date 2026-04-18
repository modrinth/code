<template>
	<div>
		<Teleport to="body">
			<div class="relative z-[100]">
				<NewModal
					ref="editAllocationModal"
					:header="formatMessage(messages.editAllocationHeader)"
					width="550px"
				>
					<form class="flex w-full flex-col gap-2" @submit.prevent="editAllocation">
						<label for="edit-allocation-name" class="font-semibold text-contrast">
							{{ formatMessage(messages.allocationNameLabel) }}
						</label>
						<StyledInput
							id="edit-allocation-name"
							ref="editAllocationInput"
							v-model="editAllocationName"
							wrapper-class="w-full"
							:maxlength="32"
							:placeholder="formatMessage(messages.allocationNamePlaceholder)"
						/>
						<div class="mb-1 mt-4 flex justify-end gap-2.5">
							<ButtonStyled>
								<button @click="editAllocationModal?.hide()">
									{{ formatMessage(commonMessages.cancelButton) }}
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<button :disabled="!editAllocationName || creatingAllocation" type="submit">
									<SaveIcon /> {{ formatMessage(messages.updateAllocationButton) }}
								</button>
							</ButtonStyled>
						</div>
					</form>
				</NewModal>

				<ConfirmModal
					ref="confirmDeleteModal"
					:title="formatMessage(messages.deleteAllocationTitle)"
					:description="deleteAllocationDescriptionText"
					:proceed-label="formatMessage(commonMessages.deleteLabel)"
					@proceed="confirmDeleteAllocation"
				/>
			</div>
		</Teleport>

		<div class="relative w-full">
			<div
				v-if="allocationsError"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
					<div class="flex flex-col items-center text-center">
						<div class="flex flex-col items-center gap-4">
							<div class="grid place-content-center rounded-full bg-bg-orange p-4">
								<IssuesIcon class="size-12 text-orange" />
							</div>
							<h1 class="m-0 mb-2 w-fit text-4xl font-semibold">
								{{ formatMessage(messages.loadNetworkErrorTitle) }}
							</h1>
						</div>
						<p class="text-md text-secondary">
							{{ formatMessage(messages.loadNetworkErrorDescription) }}
							<span class="break-all font-mono">{{
								allocationsError?.message ?? formatMessage(commonMessages.unknownLabel)
							}}</span>
						</p>
						<ButtonStyled size="large" color="brand" @click="() => refetchAllocations()">
							<button class="mt-6 !w-full">
								{{ formatMessage(commonMessages.retryButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
			<div v-else-if="data" class="flex h-full w-full flex-col justify-between gap-4">
				<div class="flex h-full flex-col gap-6">
					<!-- Allocations section -->
					<div class="flex flex-col gap-2.5">
						<span class="text-lg font-semibold text-contrast">{{
							formatMessage(messages.allocationsSectionTitle)
						}}</span>

						<div class="flex w-full flex-col items-center justify-start gap-2 sm:flex-row">
							<StyledInput
								v-model="createAllocationName"
								wrapper-class="grow max-w-[400px]"
								:maxlength="32"
								:placeholder="formatMessage(messages.allocationNamePlaceholder)"
							/>

							<ButtonStyled color="brand">
								<button
									v-tooltip="
										!createAllocationName ? formatMessage(messages.createAllocationTooltip) : ''
									"
									:disabled="!createAllocationName || creatingAllocation"
									@click="addNewAllocation"
								>
									<PlusIcon />
									<span>{{ formatMessage(messages.createAllocationButton) }}</span>
								</button>
							</ButtonStyled>
						</div>

						<Table :columns="allocationColumns" :data="allocationRows" row-key="port">
							<template #cell-name="{ row }">
								<TagItem v-if="row.primary" class="!font-medium">{{
									formatMessage(messages.primaryAllocationLabel)
								}}</TagItem>
								<span v-else class="font-semibold">{{ row.name }}</span>
							</template>
							<template #cell-port="{ row }">
								<span class="font-medium">{{ row.port }}</span>
							</template>
							<template #cell-actions="{ row }">
								<div class="flex items-center justify-end gap-2">
									<ButtonStyled icon-only type="transparent" circular>
										<button @click="copyText(`${serverIP}:${row.port}`)">
											<CopyIcon />
										</button>
									</ButtonStyled>
									<template v-if="!row.primary">
										<ButtonStyled icon-only type="transparent" circular>
											<button @click="showEditAllocationModal(row.port)">
												<PencilIcon />
											</button>
										</ButtonStyled>
										<ButtonStyled icon-only type="outlined" circular color="red">
											<button @click="showConfirmDeleteModal(row.port)">
												<TrashIcon />
											</button>
										</ButtonStyled>
									</template>
								</div>
							</template>
						</Table>
						<span>{{ formatMessage(messages.allocationsHelpText) }}</span>
					</div>

					<!-- DNS records section -->
					<div class="flex flex-col gap-2.5">
						<label for="user-domain" class="flex flex-col gap-2">
							<span class="text-lg font-semibold text-contrast">{{
								formatMessage(messages.dnsRecordsTitle)
							}}</span>
						</label>
						<div class="flex w-full flex-col items-center justify-start gap-2 sm:flex-row">
							<StyledInput
								id="user-domain"
								v-model="userDomain"
								wrapper-class="grow max-w-[400px]"
								:maxlength="64"
								:placeholder="exampleDomain"
							/>

							<ButtonStyled>
								<button
									class="!w-full sm:!w-auto"
									:disabled="userDomain == ''"
									@click="exportDnsRecords"
								>
									<UploadIcon />
									<span>{{ formatMessage(messages.exportDnsButton) }}</span>
								</button>
							</ButtonStyled>
						</div>

						<Table :columns="dnsColumns" :data="dnsRecords">
							<template #cell-type="{ row }">
								<TagItem
									v-if="row.type === 'SRV'"
									class="border !border-solid border-purple bg-highlight-purple !font-medium"
									:style="`--_color: var(--color-purple)`"
								>
									{{ row.type }}
								</TagItem>
								<TagItem
									v-else
									class="border !border-solid border-blue bg-highlight-blue !font-medium"
									:style="`--_color: var(--color-blue)`"
								>
									{{ row.type }}
								</TagItem>
							</template>
							<template #cell-name="{ row }">
								<span
									class="block cursor-pointer truncate pr-8 font-semibold"
									@click="copyText(row.name)"
								>
									{{ row.name }}
								</span>
							</template>
							<template #cell-content="{ row }">
								<span
									class="block cursor-pointer truncate pr-8 font-semibold"
									@click="copyText(row.content)"
								>
									{{ row.content }}
								</span>
							</template>
						</Table>

						<span>{{ formatMessage(messages.dnsRecordsHelpText) }}</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	CopyIcon,
	IssuesIcon,
	PencilIcon,
	PlusIcon,
	SaveIcon,
	TrashIcon,
	UploadIcon,
} from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import { ButtonStyled, ConfirmModal, NewModal, StyledInput, Table, TagItem } from '#ui/components'
import type { TableColumn } from '#ui/components/base'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const messages = defineMessages({
	editAllocationHeader: {
		id: 'server.settings.network.edit-allocation.title',
		defaultMessage: 'Edit allocation',
	},
	allocationNameLabel: {
		id: 'server.settings.network.allocation-name.label',
		defaultMessage: 'Name',
	},
	allocationNamePlaceholder: {
		id: 'server.settings.network.allocation-name.placeholder',
		defaultMessage: 'e.g. Secondary allocation',
	},
	updateAllocationButton: {
		id: 'server.settings.network.update-allocation',
		defaultMessage: 'Update allocation',
	},
	deleteAllocationTitle: {
		id: 'server.settings.network.delete-allocation.title',
		defaultMessage: 'Deleting allocation',
	},
	deleteAllocationDescription: {
		id: 'server.settings.network.delete-allocation.description',
		defaultMessage:
			'You are deleting the allocation on port {port}. This cannot be reserved again. Are you sure you want to proceed?',
	},
	loadNetworkErrorTitle: {
		id: 'server.settings.network.error.load.title',
		defaultMessage: 'Failed to load network settings',
	},
	loadNetworkErrorDescription: {
		id: 'server.settings.network.error.load.description',
		defaultMessage: "We couldn't load your server's network settings. Here's what we know:",
	},
	allocationsSectionTitle: {
		id: 'server.settings.network.allocations.title',
		defaultMessage: 'Allocations',
	},
	createAllocationTooltip: {
		id: 'server.settings.network.create-allocation.tooltip',
		defaultMessage: 'Enter a name to create an allocation',
	},
	createAllocationButton: {
		id: 'server.settings.network.create-allocation',
		defaultMessage: 'Create allocation',
	},
	primaryAllocationLabel: {
		id: 'server.settings.network.primary-allocation',
		defaultMessage: 'Primary',
	},
	primaryAllocationRowName: {
		id: 'server.settings.network.primary-allocation-row-name',
		defaultMessage: 'Primary allocation',
	},
	allocationsHelpText: {
		id: 'server.settings.network.allocations.help',
		defaultMessage:
			'Create additional ports for internet-facing features like map viewers or voice chat mods.',
	},
	dnsRecordsTitle: {
		id: 'server.settings.network.dns.title',
		defaultMessage: 'DNS records',
	},
	exportDnsButton: {
		id: 'server.settings.network.dns.export',
		defaultMessage: 'Export',
	},
	dnsRecordsHelpText: {
		id: 'server.settings.network.dns.help',
		defaultMessage: 'Set up your personal domain to connect to your server via custom DNS records.',
	},
	columnName: {
		id: 'server.settings.network.column.name',
		defaultMessage: 'Name',
	},
	columnPort: {
		id: 'server.settings.network.column.port',
		defaultMessage: 'Port',
	},
	columnActions: {
		id: 'server.settings.network.column.actions',
		defaultMessage: 'Actions',
	},
	columnRecordType: {
		id: 'server.settings.network.column.record-type',
		defaultMessage: 'Type',
	},
	columnRecordName: {
		id: 'server.settings.network.column.record-name',
		defaultMessage: 'Name',
	},
	columnRecordContent: {
		id: 'server.settings.network.column.record-content',
		defaultMessage: 'Content',
	},
	allocationReservedTitle: {
		id: 'server.settings.network.success.allocation-reserved.title',
		defaultMessage: 'Allocation reserved',
	},
	allocationReservedText: {
		id: 'server.settings.network.success.allocation-reserved.text',
		defaultMessage: 'Your allocation has been reserved.',
	},
	allocationRemovedTitle: {
		id: 'server.settings.network.success.allocation-removed.title',
		defaultMessage: 'Allocation removed',
	},
	allocationRemovedText: {
		id: 'server.settings.network.success.allocation-removed.text',
		defaultMessage: 'Your allocation has been removed.',
	},
	allocationUpdatedTitle: {
		id: 'server.settings.network.success.allocation-updated.title',
		defaultMessage: 'Allocation updated',
	},
	allocationUpdatedText: {
		id: 'server.settings.network.success.allocation-updated.text',
		defaultMessage: 'Your allocation has been updated.',
	},
	textCopiedTitle: {
		id: 'server.settings.network.success.text-copied.title',
		defaultMessage: 'Text copied',
	},
	textCopiedText: {
		id: 'server.settings.network.success.text-copied.text',
		defaultMessage: '{text} has been copied to your clipboard',
	},
})
const { server, serverId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const data = server

const serverIP = ref(data?.value?.net?.ip ?? '')
const serverPrimaryPort = ref(data?.value?.net?.port ?? 0)
const userDomain = ref('')
const exampleDomain = 'play.example.com'

const {
	data: allocationsData,
	error: allocationsError,
	refetch: refetchAllocations,
} = useQuery({
	queryKey: ['servers', 'allocations', serverId] as const,
	queryFn: () => client.archon.servers_v0.getAllocations(serverId),
})
const allocations = allocationsData

const allocationColumns = computed<TableColumn[]>(() => [
	{ key: 'name', label: formatMessage(messages.columnName), width: '40%' },
	{ key: 'port', label: formatMessage(messages.columnPort) },
	{ key: 'actions', label: formatMessage(messages.columnActions), width: '33%', align: 'right' },
])

const allocationRows = computed(() => {
	const primary = {
		name: formatMessage(messages.primaryAllocationRowName),
		port: serverPrimaryPort.value,
		primary: true,
	}
	const extra = (allocations.value ?? []).map((a) => ({
		name: a.name,
		port: a.port,
		primary: false,
	}))
	return [primary, ...extra]
})

const dnsColumns = computed<TableColumn[]>(() => [
	{ key: 'type', label: formatMessage(messages.columnRecordType), width: '20%' },
	{ key: 'name', label: formatMessage(messages.columnRecordName), width: '35%' },
	{ key: 'content', label: formatMessage(messages.columnRecordContent) },
])

const deleteAllocationDescriptionText = computed(() =>
	allocationToDelete.value != null
		? formatMessage(messages.deleteAllocationDescription, { port: allocationToDelete.value })
		: '',
)

const editAllocationModal = ref<typeof NewModal>()
const confirmDeleteModal = ref<typeof ConfirmModal>()
const editAllocationInput = ref<HTMLInputElement | null>(null)
const createAllocationName = ref('')
const editAllocationName = ref('')
const newAllocationPort = ref(0)
const allocationToDelete = ref<number | null>(null)
const creatingAllocation = ref(false)

const addNewAllocation = async () => {
	if (!createAllocationName.value) return
	creatingAllocation.value = true

	try {
		await client.archon.servers_v0.reserveAllocation(serverId, createAllocationName.value)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] })

		createAllocationName.value = ''

		addNotification({
			type: 'success',
			title: formatMessage(messages.allocationReservedTitle),
			text: formatMessage(messages.allocationReservedText),
		})
	} catch (error) {
		console.error('Failed to reserve new allocation:', error)
	} finally {
		creatingAllocation.value = false
	}
}

const showEditAllocationModal = (port: number) => {
	newAllocationPort.value = port
	editAllocationName.value = allocations.value?.find((a) => a.port === port)?.name ?? ''
	editAllocationModal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			editAllocationInput.value?.focus()
		}, 100)
	})
}

const showConfirmDeleteModal = (port: number) => {
	allocationToDelete.value = port
	confirmDeleteModal.value?.show()
}

const confirmDeleteAllocation = async () => {
	if (allocationToDelete.value === null) return

	await client.archon.servers_v0.deleteAllocation(serverId, allocationToDelete.value)
	await queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] })

	addNotification({
		type: 'success',
		title: formatMessage(messages.allocationRemovedTitle),
		text: formatMessage(messages.allocationRemovedText),
	})

	allocationToDelete.value = null
}

const editAllocation = async () => {
	if (!editAllocationName.value) return
	creatingAllocation.value = true

	try {
		await client.archon.servers_v0.updateAllocation(
			serverId,
			newAllocationPort.value,
			editAllocationName.value,
		)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] })

		editAllocationModal.value?.hide()
		editAllocationName.value = ''

		addNotification({
			type: 'success',
			title: formatMessage(messages.allocationUpdatedTitle),
			text: formatMessage(messages.allocationUpdatedText),
		})
	} catch (error) {
		console.error('Failed to reserve new allocation:', error)
	} finally {
		creatingAllocation.value = false
	}
}

const dnsRecords = computed(() => {
	const domain = userDomain.value === '' ? exampleDomain : userDomain.value
	return [
		{
			type: 'A',
			name: `${domain}`,
			content: data.value?.net?.ip ?? '',
		},
		{
			type: 'SRV',
			name: `_minecraft._tcp.${domain}`,
			content: `0 10 ${data.value?.net?.port} ${domain}`,
		},
	]
})

type DnsRecord = {
	type: string
	name: string
	content: string
}

const exportDnsRecords = () => {
	const records = dnsRecords.value.reduce(
		(acc, record) => {
			const type = record.type
			if (!acc[type]) {
				acc[type] = []
			}
			acc[type].push(record)
			return acc
		},
		{} as Record<string, DnsRecord[]>,
	)

	const text = Object.entries(records)
		.map(([type, records]) => {
			return `; ${type} Records\n${records.map((record) => `${record.name}.\t1\tIN\t${record.type} ${record.content}${record.type === 'SRV' ? '.' : ''}`).join('\n')}\n`
		})
		.join('\n')
	const blob = new Blob([text], { type: 'text/plain' })
	const a = document.createElement('a')
	a.href = window.URL.createObjectURL(blob)
	a.download = `${userDomain.value}.txt`
	a.click()
	a.remove()
}

const copyText = (text: string) => {
	navigator.clipboard.writeText(text)
	addNotification({
		type: 'success',
		title: formatMessage(messages.textCopiedTitle),
		text: formatMessage(messages.textCopiedText, { text }),
	})
}
</script>
