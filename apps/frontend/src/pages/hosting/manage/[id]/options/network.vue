<template>
	<div class="contents">
		<NewModal ref="newAllocationModal" header="New allocation">
			<form class="flex flex-col gap-2 md:w-[600px]" @submit.prevent="addNewAllocation">
				<label for="new-allocation-name" class="font-semibold text-contrast"> Name </label>
				<input
					id="new-allocation-name"
					ref="newAllocationInput"
					v-model="newAllocationName"
					type="text"
					class="bg-bg-input w-full rounded-lg p-4"
					maxlength="32"
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
				<input
					id="edit-allocation-name"
					ref="editAllocationInput"
					v-model="newAllocationName"
					type="text"
					class="bg-bg-input w-full rounded-lg p-4"
					maxlength="32"
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
				v-if="isError"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
					<div class="flex flex-col items-center text-center">
						<div class="flex flex-col items-center gap-4">
							<div class="grid place-content-center rounded-full bg-bg-orange p-4">
								<IssuesIcon class="size-12 text-orange" />
							</div>
							<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load network settings</h1>
						</div>
						<p class="text-lg text-secondary">
							We couldn't load your server's network settings. Here's what we know:
						</p>
						<p>
							<span class="break-all font-mono">{{ error?.message ?? 'Unknown error' }}</span>
						</p>
						<ButtonStyled size="large" color="brand" @click="refetch">
							<button class="mt-6 !w-full">Retry</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
			<div v-else-if="isLoading" class="flex h-full w-full items-center justify-center">
				<div class="text-secondary">Loading network settings...</div>
			</div>
			<div v-else class="flex h-full w-full flex-col justify-between gap-4">
				<div class="flex h-full flex-col">
					<!-- Subdomain section -->
					<div class="card flex flex-col gap-4">
						<div class="flex w-full flex-col items-center justify-between gap-4 sm:flex-row">
							<label for="user-domain" class="flex flex-col gap-2">
								<span class="text-lg font-bold text-contrast">Generated DNS records</span>
								<span>
									Set up your personal domain to connect to your server via custom DNS records.
								</span>
							</label>

							<ButtonStyled>
								<button
									class="!w-full sm:!w-auto"
									:disabled="userDomain == ''"
									@click="exportDnsRecords"
								>
									<UploadIcon />
									<span>Export DNS records</span>
								</button>
							</ButtonStyled>
						</div>

						<input
							id="user-domain"
							v-model="userDomain"
							class="w-full md:w-[50%]"
							maxlength="64"
							minlength="1"
							type="text"
							:placeholder="exampleDomain"
						/>

						<div
							class="flex max-w-full flex-none overflow-auto rounded-xl bg-table-alternateRow px-4 py-2"
						>
							<table
								class="w-full flex-none border-collapse truncate rounded-lg border-2 border-gray-300"
							>
								<tbody class="w-full">
									<tr v-for="record in dnsRecords" :key="record.content" class="w-full">
										<td class="w-1/6 py-3 pr-4 md:w-1/5 md:pr-8 lg:w-1/4 lg:pr-12">
											<div class="flex flex-col gap-1" @click="copyText(record.type)">
												<span
													class="text-md font-bold tracking-wide text-contrast hover:cursor-pointer"
												>
													{{ record.type }}
												</span>
												<span class="text-xs text-secondary">Type</span>
											</div>
										</td>
										<td class="w-2/6 py-3 md:w-1/3">
											<div class="flex flex-col gap-1" @click="copyText(record.name)">
												<span
													class="text-md truncate font-bold tracking-wide text-contrast hover:cursor-pointer"
												>
													{{ record.name }}
												</span>
												<span class="text-xs text-secondary">Name</span>
											</div>
										</td>
										<td class="w-3/6 py-3 pl-4 md:w-5/12 lg:w-5/12">
											<div class="flex flex-col gap-1" @click="copyText(record.content)">
												<span
													class="text-md w-fit truncate font-bold tracking-wide text-contrast hover:cursor-pointer"
												>
													{{ record.content }}
												</span>
												<span class="text-xs text-secondary">Content</span>
											</div>
										</td>
									</tr>
								</tbody>
							</table>
						</div>

						<div class="flex items-center gap-2">
							<InfoIcon class="hidden sm:block" />
							<span class="text-sm text-secondary">
								You must own your own domain to use this feature.
							</span>
						</div>
					</div>

					<!-- Allocations section -->
					<div class="card flex flex-col gap-4">
						<div class="flex w-full flex-col items-center justify-between gap-4 sm:flex-row">
							<div class="flex flex-col gap-2">
								<span class="text-lg font-bold text-contrast">Allocations</span>
								<span>
									Configure additional ports for internet-facing features like map viewers or voice
									chat mods.
								</span>
							</div>

							<ButtonStyled type="standard" @click="showNewAllocationModal">
								<button class="!w-full sm:!w-auto">
									<PlusIcon />
									<span>New allocation</span>
								</button>
							</ButtonStyled>
						</div>

						<div class="flex w-full flex-col overflow-hidden rounded-xl bg-table-alternateRow p-4">
							<!-- Primary allocation -->
							<div class="flex flex-col justify-between gap-2 sm:flex-row sm:items-center">
								<span class="text-md font-bold tracking-wide text-contrast">
									Primary allocation
								</span>

								<CopyCode :text="`${server.net.ip}:${server.net.port}`" />
							</div>
						</div>

						<div
							v-if="allocations && allocations.length > 0"
							class="flex w-full flex-col gap-4 overflow-hidden rounded-xl bg-table-alternateRow p-4"
						>
							<div
								v-for="allocation in allocations"
								:key="allocation.port"
								class="border-border flex flex-col justify-between gap-4 sm:flex-row sm:items-center"
							>
								<div class="flex flex-row items-center gap-4">
									<VersionIcon class="h-7 w-7 flex-none rotate-90" />
									<div class="flex w-[20rem] flex-col justify-between sm:flex-row sm:items-center">
										<div class="flex flex-col gap-1">
											<span class="text-md font-bold tracking-wide text-contrast">
												{{ allocation.name }}
											</span>
											<span class="hidden text-xs text-secondary sm:block">Name</span>
										</div>
										<div class="flex flex-col gap-1">
											<span
												class="text-md w-10 tracking-wide text-secondary sm:font-bold sm:text-contrast"
											>
												{{ allocation.port }}
											</span>
											<span class="hidden text-xs text-secondary sm:block">Port</span>
										</div>
									</div>
								</div>

								<div class="flex w-full flex-row items-center gap-2 sm:w-auto">
									<CopyCode :text="`${server.net.ip}:${allocation.port}`" />
									<ButtonStyled icon-only>
										<button
											class="!w-full sm:!w-auto"
											@click="showEditAllocationModal(allocation.port)"
										>
											<EditIcon />
										</button>
									</ButtonStyled>
									<ButtonStyled icon-only color="red">
										<button
											class="!w-full sm:!w-auto"
											@click="showConfirmDeleteModal(allocation.port)"
										>
											<TrashIcon />
										</button>
									</ButtonStyled>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
			<UnsavedChangesPopup
				:original="originalValues"
				:modified="modifiedValues"
				:saving="isSaving"
				@save="saveNetwork"
				@reset="resetNetwork"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	EditIcon,
	InfoIcon,
	IssuesIcon,
	PlusIcon,
	SaveIcon,
	TrashIcon,
	UploadIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	CopyCode,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	NewModal,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server, serverId } = injectModrinthServerContext()

// Fetch allocations
const {
	data: allocations,
	isLoading,
	isError,
	error,
	refetch,
} = useQuery({
	queryKey: ['server-allocations', serverId],
	queryFn: () => client.archon.settings_v0.getAllocations(serverId),
})

const isSaving = ref(false)
const serverSubdomain = ref(server.value.net?.domain ?? '')
const userDomain = ref('')
const exampleDomain = 'play.example.com'

const newAllocationModal = ref<typeof NewModal>()
const editAllocationModal = ref<typeof NewModal>()
const confirmDeleteModal = ref<typeof ConfirmModal>()
const newAllocationInput = ref<HTMLInputElement | null>(null)
const editAllocationInput = ref<HTMLInputElement | null>(null)
const newAllocationName = ref('')
const newAllocationPort = ref(0)
const allocationToDelete = ref<number | null>(null)

const subdomainError = computed(() => {
	if (serverSubdomain.value.length === 0) return null
	if (serverSubdomain.value.length < 5) return 'Subdomain must be at least 5 characters long.'
	if (!/^[a-zA-Z0-9-]+$/.test(serverSubdomain.value))
		return 'Subdomain can only contain alphanumeric characters and dashes.'
	return null
})

const isValid = computed(() => !subdomainError.value)

const originalValues = computed(() => ({
	subdomain: server.value.net?.domain ?? '',
}))

const modifiedValues = computed(() => ({
	subdomain: serverSubdomain.value,
}))

// Mutations
const reserveAllocationMutation = useMutation({
	mutationFn: (name: string) => client.archon.settings_v0.reserveAllocation(serverId, name),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['server-allocations', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to reserve allocation',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

const updateAllocationMutation = useMutation({
	mutationFn: ({ port, name }: { port: number; name: string }) =>
		client.archon.settings_v0.updateAllocation(serverId, port, name),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['server-allocations', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to update allocation',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

const deleteAllocationMutation = useMutation({
	mutationFn: (port: number) => client.archon.settings_v0.deleteAllocation(serverId, port),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['server-allocations', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to delete allocation',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

const updateSubdomainMutation = useMutation({
	mutationFn: async (subdomain: string) => {
		const available = await client.archon.settings_v0.checkSubdomainAvailability(subdomain)
		if (!available) throw new Error('Subdomain not available')
		await client.archon.settings_v0.updateSubdomain(serverId, subdomain)
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to update subdomain',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

async function addNewAllocation() {
	if (!newAllocationName.value) return

	try {
		await reserveAllocationMutation.mutateAsync(newAllocationName.value)
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

function showNewAllocationModal() {
	newAllocationName.value = ''
	newAllocationModal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			newAllocationInput.value?.focus()
		}, 100)
	})
}

function showEditAllocationModal(port: number) {
	newAllocationPort.value = port
	editAllocationModal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			editAllocationInput.value?.focus()
		}, 100)
	})
}

function showConfirmDeleteModal(port: number) {
	allocationToDelete.value = port
	confirmDeleteModal.value?.show()
}

async function confirmDeleteAllocation() {
	if (allocationToDelete.value === null) return

	await deleteAllocationMutation.mutateAsync(allocationToDelete.value)
	addNotification({
		type: 'success',
		title: 'Allocation removed',
		text: 'Your allocation has been removed.',
	})
	allocationToDelete.value = null
}

async function editAllocation() {
	if (!newAllocationName.value) return

	try {
		await updateAllocationMutation.mutateAsync({
			port: newAllocationPort.value,
			name: newAllocationName.value,
		})
		editAllocationModal.value?.hide()
		newAllocationName.value = ''
		addNotification({
			type: 'success',
			title: 'Allocation updated',
			text: 'Your allocation has been updated.',
		})
	} catch (error) {
		console.error('Failed to update allocation:', error)
	}
}

async function saveNetwork() {
	if (!isValid.value) return

	try {
		isSaving.value = true
		if (serverSubdomain.value !== server.value.net?.domain) {
			await updateSubdomainMutation.mutateAsync(serverSubdomain.value)
		}
		addNotification({
			type: 'success',
			title: 'Server settings updated',
			text: 'Your server settings were successfully changed.',
		})
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update server settings',
			text: 'An error occurred while attempting to update your server settings.',
		})
	} finally {
		isSaving.value = false
	}
}

function resetNetwork() {
	serverSubdomain.value = server.value.net?.domain ?? ''
}

const dnsRecords = computed(() => {
	const domain = userDomain.value === '' ? exampleDomain : userDomain.value
	return [
		{
			type: 'A',
			name: `${domain}`,
			content: server.value.net?.ip ?? '',
		},
		{
			type: 'SRV',
			name: `_minecraft._tcp.${domain}`,
			content: `0 10 ${server.value.net?.port} ${domain}`,
		},
	]
})

function exportDnsRecords() {
	const records = dnsRecords.value.reduce(
		(acc, record) => {
			const type = record.type
			if (!acc[type]) {
				acc[type] = []
			}
			acc[type].push(record)
			return acc
		},
		{} as Record<string, typeof dnsRecords.value>,
	)

	const text = Object.entries(records)
		.map(([type, records]) => {
			return `; ${type} Records\n${records.map((record) => `${record.name}.	1	IN	${record.type} ${record.content}${record.type === 'SRV' ? '.' : ''}`).join('\n')}\n`
		})
		.join('\n')
	const blob = new Blob([text], { type: 'text/plain' })
	const a = document.createElement('a')
	a.href = window.URL.createObjectURL(blob)
	a.download = `${userDomain.value}.txt`
	a.click()
	a.remove()
}

function copyText(text: string) {
	navigator.clipboard.writeText(text)
	addNotification({
		type: 'success',
		title: 'Text copied',
		text: `${text} has been copied to your clipboard`,
	})
}
</script>
