<template>
	<div class="contents">
		<NewModal ref="newAllocationModal" header="New allocation">
			<form class="flex flex-col gap-2 md:w-[600px]" @submit.prevent="addNewAllocation">
				<label for="new-allocation-name" class="font-semibold text-contrast"> Name </label>
				<StyledInput
					id="new-allocation-name"
					ref="newAllocationInput"
					v-model="newAllocationName"
					wrapper-class="w-full"
					:maxlength="32"
					placeholder="e.g. Secondary allocation"
				/>
				<div class="mb-1 mt-4 flex justify-start gap-4">
					<ButtonStyled color="brand">
						<button :disabled="!newAllocationName" type="submit">
							<PlusIcon /> Create allocation
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="newAllocationModal?.hide()">Cancel</button>
					</ButtonStyled>
				</div>
			</form>
		</NewModal>

		<NewModal ref="editAllocationModal" header="Edit allocation">
			<form class="flex flex-col gap-2 md:w-[600px]" @submit.prevent="editAllocation">
				<label for="edit-allocation-name" class="font-semibold text-contrast"> Name </label>
				<StyledInput
					id="edit-allocation-name"
					ref="editAllocationInput"
					v-model="newAllocationName"
					wrapper-class="w-full"
					:maxlength="32"
					placeholder="e.g. Secondary allocation"
				/>
				<div class="mb-1 mt-4 flex justify-start gap-4">
					<ButtonStyled color="brand">
						<button :disabled="!newAllocationName" type="submit">
							<SaveIcon /> Update allocation
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="editAllocationModal?.hide()">Cancel</button>
					</ButtonStyled>
				</div>
			</form>
		</NewModal>

		<ConfirmModal
			ref="confirmDeleteModal"
			title="Deleting allocation"
			:description="`You are deleting the allocation ${allocationToDelete}. This cannot be reserved again. Are you sure you want to proceed?`"
			proceed-label="Delete"
			@proceed="confirmDeleteAllocation"
		/>

		<div class="relative h-full w-full overflow-y-auto">
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
							<h1 class="m-0 mb-2 w-fit text-4xl font-semibold">Failed to load network settings</h1>
						</div>
						<p class="text-md text-secondary">
							We couldn't load your server's network settings. Here's what we know:
							<span class="break-all font-mono">{{
								allocationsError?.message ?? 'Unknown error'
							}}</span>
						</p>
						<ButtonStyled size="large" color="brand" @click="() => refetchAllocations()">
							<button class="mt-6 !w-full">Retry</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
			<div v-else-if="data" class="flex h-full w-full flex-col justify-between gap-4">
				<div class="card flex h-full flex-col gap-6">
					<!-- Allocations section -->
					<div class="flex flex-col gap-2.5">
						<span class="text-md font-semibold text-contrast">Allocations</span>

						<div class="flex w-full flex-col items-center justify-start gap-2 sm:flex-row">
							<StyledInput
								v-model="allocationSearch"
								wrapper-class="grow max-w-[400px]"
								:maxlength="64"
								placeholder="Search allocations..."
							/>

							<ButtonStyled color="brand" @click="showNewAllocationModal">
								<button class="!w-full max-w-20">
									<PlusIcon />
									<span>Add</span>
								</button>
							</ButtonStyled>
						</div>

						<Table :columns="allocationColumns" :data="allocationRows" row-key="port">
							<template #cell-name="{ row }">
								<TagItem v-if="row.primary" class="!font-medium">Primary</TagItem>
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
						<span>
							Create additional ports for internet-facing features like map viewers or voice chat
							mods.
						</span>
					</div>

					<!-- DNS records section -->
					<div class="flex flex-col gap-2.5">
						<label for="user-domain" class="flex flex-col gap-2">
							<span class="text-md font-semibold text-contrast">DNS records</span>
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
									<span>Export</span>
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

						<span>
							Set up your personal domain to connect to your server via custom DNS records. 
						</span>
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
import type { TableColumn } from '@modrinth/ui'
import {
	ButtonStyled,
	ConfirmModal,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	NewModal,
	StyledInput,
	Table,
	TagItem,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { nextTick, ref } from 'vue'

const { addNotification } = injectNotificationManager()
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

const allocationColumns: TableColumn[] = [
	{ key: 'name', label: 'Name' },
	{ key: 'port', label: 'Port' },
	{ key: 'actions', label: 'Actions', width: '33%', align: 'right' },
]

const allocationSearch = ref('')

const allocationRows = computed(() => {
	const primary = {
		name: 'Primary allocation',
		port: serverPrimaryPort.value,
		primary: true,
	}
	const extra = (allocations.value ?? []).map((a) => ({
		name: a.name,
		port: a.port,
		primary: false,
	}))
	const all = [primary, ...extra]
	const query = allocationSearch.value.toLowerCase().trim()
	if (!query) return all
	return all.filter(
		(row) => row.name.toLowerCase().includes(query) || String(row.port).includes(query),
	)
})

const dnsColumns: TableColumn[] = [
	{ key: 'type', label: 'Type', width: '15%' },
	{ key: 'name', label: 'Name', width: '35%' },
	{ key: 'content', label: 'Content' },
]

const newAllocationModal = ref<typeof NewModal>()
const editAllocationModal = ref<typeof NewModal>()
const confirmDeleteModal = ref<typeof ConfirmModal>()
const newAllocationInput = ref<HTMLInputElement | null>(null)
const editAllocationInput = ref<HTMLInputElement | null>(null)
const newAllocationName = ref('')
const newAllocationPort = ref(0)
const allocationToDelete = ref<number | null>(null)

const addNewAllocation = async () => {
	if (!newAllocationName.value) return

	try {
		await client.archon.servers_v0.reserveAllocation(serverId, newAllocationName.value)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] })

		newAllocationModal.value?.hide()
		newAllocationName.value = ''

		addNotification({
			type: 'success',
			title: 'Allocation reserved',
			text: 'Your allocation has been reserved.',
		})
	} catch (error) {
		console.error('Failed to reserve new allocation:', error)
	}
}

const showNewAllocationModal = () => {
	newAllocationName.value = ''
	newAllocationModal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			newAllocationInput.value?.focus()
		}, 100)
	})
}

const showEditAllocationModal = (port: number) => {
	newAllocationPort.value = port
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
		title: 'Allocation removed',
		text: 'Your allocation has been removed.',
	})

	allocationToDelete.value = null
}

const editAllocation = async () => {
	if (!newAllocationName.value) return

	try {
		await client.archon.servers_v0.updateAllocation(
			serverId,
			newAllocationPort.value,
			newAllocationName.value,
		)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] })

		editAllocationModal.value?.hide()
		newAllocationName.value = ''

		addNotification({
			type: 'success',
			title: 'Allocation updated',
			text: 'Your allocation has been updated.',
		})
	} catch (error) {
		console.error('Failed to reserve new allocation:', error)
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
		{} as Record<string, any[]>,
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
		title: 'Text copied',
		text: `${text} has been copied to your clipboard`,
	})
}
</script>
