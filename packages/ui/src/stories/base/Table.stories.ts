import { EditIcon, MoreVerticalIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import Badge from '../../components/base/Badge.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import OverflowMenu from '../../components/base/OverflowMenu.vue'
import Table from '../../components/base/Table.vue'

interface User {
	id: string
	name: string
	email: string
	status: 'active' | 'inactive' | 'pending'
	role: string
}

const sampleUsers: User[] = [
	{ id: '1', name: 'John Doe', email: 'john@example.com', status: 'active', role: 'Admin' },
	{ id: '2', name: 'Jane Smith', email: 'jane@example.com', status: 'inactive', role: 'User' },
	{ id: '3', name: 'Bob Johnson', email: 'bob@example.com', status: 'pending', role: 'Editor' },
	{ id: '4', name: 'Alice Brown', email: 'alice@example.com', status: 'active', role: 'User' },
	{
		id: '5',
		name: 'Charlie Wilson',
		email: 'charlie@example.com',
		status: 'active',
		role: 'Admin',
	},
]

const meta = {
	title: 'Base/Table',
	// @ts-ignore - Generic component
	component: Table,
} satisfies Meta<typeof Table>

export default meta

export const Default: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role' },
			]
			const data = sampleUsers
			return { columns, data }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data" />
		`,
	}),
}

export const WithSelection: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role' },
			]
			const data = sampleUsers
			const selectedIds = ref<string[]>([])
			return { columns, data, selectedIds }
		},
		template: /* html */ `
			<div class="space-y-4">
				<Table
					:columns="columns"
					:data="data"
					show-selection
					row-key="id"
					v-model:selected-ids="selectedIds"
				/>
				<p class="text-secondary">Selected IDs: {{ selectedIds.join(', ') || 'None' }}</p>
			</div>
		`,
	}),
}

export const WithSorting: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name', enableSorting: true },
				{ key: 'email', label: 'Email', enableSorting: true },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role', enableSorting: true },
			]
			const data = sampleUsers
			const sortColumn = ref<string | undefined>('name')
			const sortDirection = ref<'asc' | 'desc'>('asc')

			function handleSort(column: string, direction: 'asc' | 'desc') {
				console.log(`Sorting by ${column} ${direction}`)
			}

			return { columns, data, sortColumn, sortDirection, handleSort }
		},
		template: /* html */ `
			<div class="space-y-4">
				<Table
					:columns="columns"
					:data="data"
					v-model:sort-column="sortColumn"
					v-model:sort-direction="sortDirection"
					@sort="handleSort"
				/>
				<p class="text-secondary">Sort: {{ sortColumn }} ({{ sortDirection }})</p>
			</div>
		`,
	}),
}

export const WithColumnAlignment: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name', align: 'left' as const },
				{ key: 'email', label: 'Email', align: 'center' as const },
				{ key: 'status', label: 'Status', align: 'center' as const },
				{ key: 'role', label: 'Role', align: 'right' as const },
			]
			const data = sampleUsers
			return { columns, data }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data" />
		`,
	}),
}

export const WithCustomCellSlots: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, Badge },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status', align: 'center' as const, width: '20%' },
				{ key: 'role', label: 'Role', width: '10%' },
			]
			const data = sampleUsers

			const statusColor = (status: string) => {
				switch (status) {
					case 'active':
						return 'green'
					case 'inactive':
						return 'red'
					case 'pending':
						return 'orange'
					default:
						return 'gray'
				}
			}

			return { columns, data, statusColor }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #cell-name="{ value, row }">
					<div class="font-semibold">{{ value }}</div>
				</template>
				<template #cell-email="{ value }">
					<a :href="'mailto:' + value" class="text-brand hover:underline">{{ value }}</a>
				</template>
				<template #cell-status="{ value }">
					<div class="flex justify-center">
						<Badge :color="statusColor(value)">{{ value }}</Badge>
					</div>
				</template>
			</Table>
		`,
	}),
}

export const WithCustomHeaderSlots: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role' },
			]
			const data = sampleUsers
			return { columns, data }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #header-name="{ column }">
					<span class="text-brand font-bold uppercase">{{ column.label }} ✨</span>
				</template>
				<template #header-status="{ column }">
					<span class="flex items-center gap-1">
						<span class="w-2 h-2 rounded-full bg-green"></span>
						{{ column.label }}
					</span>
				</template>
			</Table>
		`,
	}),
}

export const WithHeaderSlot: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, ButtonStyled },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role' },
			]
			const data = sampleUsers

			return { columns, data }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #header>
					<div class="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
						<div class="text-lg font-semibold text-contrast">Team Members</div>
						<div class="flex items-center gap-2">
							<ButtonStyled color="brand">
								<button type="button">Invite member</button>
							</ButtonStyled>
						</div>
					</div>
				</template>
			</Table>
		`,
	}),
}

export const WithActionsColumn: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, ButtonStyled, EditIcon, TrashIcon },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'role', label: 'Role' },
				{ key: 'actions', label: 'Actions', align: 'right' as const },
			]
			const data = sampleUsers

			function handleEdit(row: User) {
				alert(`Edit user: ${row.name}`)
			}

			function handleDelete(row: User) {
				alert(`Delete user: ${row.name}`)
			}

			return { columns, data, handleEdit, handleDelete }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #cell-actions="{ row }">
					<div class="flex items-center justify-end gap-2">
						<ButtonStyled color="brand" type="transparent" @click="handleEdit(row)">
							<button class="flex items-center gap-1">
								<EditIcon class="size-4" />
								Edit
							</button>
						</ButtonStyled>
						<ButtonStyled color="red" type="transparent" @click="handleDelete(row)">
							<button class="flex items-center gap-1">
								<TrashIcon class="size-4" />
								Delete
							</button>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		`,
	}),
}

export const FullFeatured: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, Badge, ButtonStyled, EditIcon, TrashIcon },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name', enableSorting: true },
				{ key: 'email', label: 'Email', enableSorting: true },
				{ key: 'status', label: 'Status', align: 'center' as const, width: '100px' },
				{ key: 'role', label: 'Role', enableSorting: true },
				{ key: 'actions', label: 'Actions', align: 'right' as const, width: '200px' },
			]
			const data = sampleUsers
			const selectedIds = ref<string[]>([])
			const sortColumn = ref<string | undefined>('name')
			const sortDirection = ref<'asc' | 'desc'>('asc')

			const statusColor = (status: string) => {
				switch (status) {
					case 'active':
						return 'green'
					case 'inactive':
						return 'red'
					case 'pending':
						return 'orange'
					default:
						return 'gray'
				}
			}

			function handleSort(column: string, direction: 'asc' | 'desc') {
				console.log(`Sorting by ${column} ${direction}`)
			}

			function handleEdit(row: User) {
				alert(`Edit user: ${row.name}`)
			}

			function handleDelete(row: User) {
				alert(`Delete user: ${row.name}`)
			}

			return {
				columns,
				data,
				selectedIds,
				sortColumn,
				sortDirection,
				statusColor,
				handleSort,
				handleEdit,
				handleDelete,
			}
		},
		template: /* html */ `
			<div class="space-y-4">
				<Table
					:columns="columns"
					:data="data"
					show-selection
					row-key="id"
					v-model:selected-ids="selectedIds"
					v-model:sort-column="sortColumn"
					v-model:sort-direction="sortDirection"
					@sort="handleSort"
				>
					<template #cell-name="{ value }">
						<span class="font-semibold">{{ value }}</span>
					</template>
					<template #cell-email="{ value }">
						<a :href="'mailto:' + value" class="text-brand hover:underline">{{ value }}</a>
					</template>
					<template #cell-status="{ value }">
						<div class="flex justify-center">
							<Badge :color="statusColor(value)">{{ value }}</Badge>
						</div>
					</template>
					<template #cell-actions="{ row }">
						<div class="flex items-center justify-end gap-2">
							<ButtonStyled color="brand" type="transparent" @click="handleEdit(row)">
								<button class="flex items-center gap-1">
									<EditIcon class="size-4" />
									Edit
								</button>
							</ButtonStyled>
							<ButtonStyled color="red" type="transparent" @click="handleDelete(row)">
								<button class="flex items-center gap-1">
									<TrashIcon class="size-4" />
									Delete
								</button>
							</ButtonStyled>
						</div>
					</template>
				</Table>
				<div class="flex gap-4 text-secondary text-sm">
					<span>Selected: {{ selectedIds.length }} items</span>
					<span>Sort: {{ sortColumn }} ({{ sortDirection }})</span>
				</div>
			</div>
		`,
	}),
}

export const VirtualizedLargeData: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, Badge },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name', enableSorting: true },
				{ key: 'email', label: 'Email', enableSorting: true },
				{ key: 'status', label: 'Status', align: 'center' as const, width: '140px' },
				{ key: 'role', label: 'Role', enableSorting: true, align: 'right' as const },
			]
			const statuses: User['status'][] = ['active', 'inactive', 'pending']
			const roles = ['Admin', 'Editor', 'Maintainer', 'Reviewer', 'User']
			const largeData = Array.from({ length: 10000 }, (_, index): User => {
				const id = String(index + 1)
				const paddedId = id.padStart(5, '0')

				return {
					id,
					name: `User ${paddedId}`,
					email: `user-${paddedId}@example.com`,
					status: statuses[index % statuses.length],
					role: roles[index % roles.length],
				}
			})
			const selectedIds = ref<string[]>([])
			const sortColumn = ref<string | undefined>('name')
			const sortDirection = ref<'asc' | 'desc'>('asc')
			const data = computed(() => {
				const sorted = [...largeData]
				const activeSortColumn = sortColumn.value

				if (!activeSortColumn) {
					return sorted
				}

				const directionFactor = sortDirection.value === 'asc' ? 1 : -1
				sorted.sort((left, right) => {
					return (
						String(left[activeSortColumn as keyof User]).localeCompare(
							String(right[activeSortColumn as keyof User]),
							undefined,
							{ numeric: true, sensitivity: 'base' },
						) * directionFactor
					)
				})

				return sorted
			})

			const statusColor = (status: string) => {
				switch (status) {
					case 'active':
						return 'green'
					case 'inactive':
						return 'red'
					case 'pending':
						return 'orange'
					default:
						return 'gray'
				}
			}

			function handleSort(column: string, direction: 'asc' | 'desc') {
				console.log(`Sorting ${largeData.length} rows by ${column} ${direction}`)
			}

			return {
				columns,
				data,
				selectedIds,
				sortColumn,
				sortDirection,
				statusColor,
				handleSort,
			}
		},
		template: /* html */ `
			<div class="space-y-4 max-h-[60vh] overflow-y-scroll">
				<Table
					:columns="columns"
					:data="data"
					show-selection
					row-key="id"
					virtualized
					:virtual-row-height="56"
					v-model:selected-ids="selectedIds"
					v-model:sort-column="sortColumn"
					v-model:sort-direction="sortDirection"
					@sort="handleSort"
				>
					<template #header>
						<div class="flex items-center justify-between gap-4">
							<div class="text-lg font-semibold text-contrast">Virtualized members</div>
							<div class="text-sm text-secondary">{{ data.length.toLocaleString() }} rows</div>
						</div>
					</template>
					<template #cell-name="{ value, index }">
						<div class="flex items-center gap-2">
							<span class="text-secondary tabular-nums">#{{ index + 1 }}</span>
							<span class="font-semibold">{{ value }}</span>
						</div>
					</template>
					<template #cell-status="{ value }">
						<div class="flex justify-center">
							<Badge :color="statusColor(value)">{{ value }}</Badge>
						</div>
					</template>
				</Table>
				<div class="flex gap-4 text-secondary text-sm">
					<span>Selected: {{ selectedIds.length }} items</span>
					<span>Sort: {{ sortColumn }} ({{ sortDirection }})</span>
				</div>
			</div>
		`,
	}),
}

export const WithOverflowMenu: StoryObj = {
	args: {},
	render: () => ({
		components: { Table, Badge, ButtonStyled, OverflowMenu, MoreVerticalIcon, EditIcon, TrashIcon },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status', align: 'center' as const, width: '20%' },
				{ key: 'role', label: 'Role' },
				{ key: 'actions', label: '', width: '68px' },
			]
			const data = sampleUsers

			const statusColor = (status: string) => {
				switch (status) {
					case 'active':
						return 'green'
					case 'inactive':
						return 'red'
					case 'pending':
						return 'orange'
					default:
						return 'gray'
				}
			}

			const getMenuOptions = (row: User) => [
				{
					id: 'edit',
					action: () => alert(`Edit user: ${row.name}`),
				},
				{
					id: 'duplicate',
					action: () => alert(`Duplicate user: ${row.name}`),
				},
				{ divider: true },
				{
					id: 'delete',
					color: 'red' as const,
					hoverFilled: true,
					action: () => alert(`Delete user: ${row.name}`),
				},
			]

			return { columns, data, statusColor, getMenuOptions }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #cell-name="{ value }">
					<span class="font-semibold">{{ value }}</span>
				</template>
				<template #cell-status="{ value }">
					<div class="flex justify-center">
						<Badge :color="statusColor(value)">{{ value }}</Badge>
					</div>
				</template>
				<template #cell-actions="{ row }">
					<div class="flex justify-end">
						<ButtonStyled circular type="transparent">
							<OverflowMenu
								:options="getMenuOptions(row)"
								aria-label="More options"
							>
								<MoreVerticalIcon aria-hidden="true" />
								<template #edit>
									<EditIcon class="size-4" aria-hidden="true" />
									Edit
								</template>
								<template #duplicate>
									<EditIcon class="size-4" aria-hidden="true" />
									Duplicate
								</template>
								<template #delete>
									<TrashIcon class="size-4" aria-hidden="true" />
									Delete
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</template>
			</Table>
		`,
	}),
}

export const EmptyState: StoryObj = {
	args: {},
	render: () => ({
		components: { Table },
		setup() {
			const columns = [
				{ key: 'name', label: 'Name' },
				{ key: 'email', label: 'Email' },
				{ key: 'status', label: 'Status' },
				{ key: 'role', label: 'Role' },
			]
			const data: User[] = []

			return { columns, data }
		},
		template: /* html */ `
			<Table :columns="columns" :data="data">
				<template #empty-state>
					<div class="flex h-64 flex-col items-center justify-center gap-2 text-center">
						<div class="font-semibold text-contrast">No members found</div>
						<div class="text-sm text-secondary">Invite a team member to get started.</div>
					</div>
				</template>
			</Table>
		`,
	}),
}
