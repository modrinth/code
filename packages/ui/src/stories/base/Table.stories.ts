import { EditIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'
import Badge from '../../components/base/Badge.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
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
				{ key: 'status', label: 'Status', align: 'center' as const },
				{ key: 'role', label: 'Role' },
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
					<Badge :color="statusColor(value)">{{ value }}</Badge>
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
				{ key: 'status', label: 'Status', align: 'center' as const },
				{ key: 'role', label: 'Role', enableSorting: true },
				{ key: 'actions', label: 'Actions', align: 'right' as const },
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
						<Badge :color="statusColor(value)">{{ value }}</Badge>
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
