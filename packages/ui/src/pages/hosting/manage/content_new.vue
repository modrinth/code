<script setup lang="ts">
import {
	CompassIcon,
	ListFilterIcon,
	SearchIcon,
	SortAscIcon,
	SortDescIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../../components/base/Combobox.vue'
import Pagination from '../../../components/base/Pagination.vue'
import ContentCard from '../../../components/instances/ContentCard.vue'
import ContentModpackCard from '../../../components/instances/ContentModpackCard.vue'
import ModpackUnlinkModal from '../../../components/instances/modals/ModpackUnlinkModal.vue'
import type {
	ContentCardProject,
	ContentCardVersion,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '../../../components/instances/types'

const modpack = {
	project: {
		id: '1',
		slug: 'cobblemon',
		title: 'Cobblemon Official Modpack',
		icon_url:
			'https://cdn.modrinth.com/data/MdwFAVRL/d65f0991eabdd23a4c0e36cbb7f34b2e7c477cf5.webp',
		description:
			'Cobblemon is a Minecraft mod for Fabric and NeoForge that lets you explore, capture, and battle Pokémon!',
		downloads: 11110000,
		followers: 5460,
	} as ContentModpackCardProject,
	version: {
		id: '1',
		version_number: '1.6.1',
		date_published: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
	} as ContentModpackCardVersion,
	owner: {
		id: '1',
		name: 'Cobbled Studios',
		type: 'organization',
		avatar_url:
			'https://cdn.modrinth.com/data/MdwFAVRL/d65f0991eabdd23a4c0e36cbb7f34b2e7c477cf5.webp',
	} as ContentOwner,
	categories: [
		{ name: 'Adventure', header: 'Categories' },
		{ name: 'Lightweight', header: 'Categories' },
		{ name: 'Multiplayer', header: 'Categories' },
	] as ContentModpackCardCategory[],
}

interface ContentItem {
	project: ContentCardProject
	version?: ContentCardVersion
	owner?: ContentOwner
	enabled: boolean
	hasUpdate?: boolean
}

const contentItems = ref<ContentItem[]>([
	{
		project: {
			id: '2',
			slug: 'emf',
			title: '[EMF] Entity Model Features',
			icon_url: 'https://cdn.modrinth.com/data/4I1XuqiY/icon.png',
		},
		version: {
			id: '2',
			version_number: '2.4.1',
			file_name: 'Entity_model_features_fabric_1.21.1-2.4.1.jar',
		},
		owner: {
			id: '2',
			name: 'Traben',
			type: 'user',
			avatar_url: 'https://cdn.modrinth.com/user/Traben/avatar.webp',
		},
		enabled: true,
		hasUpdate: true,
	},
	{
		project: {
			id: '3',
			slug: 'etf',
			title: '[EMF] Entity Texture Features',
			icon_url: 'https://cdn.modrinth.com/data/BVzZfTc1/icon.png',
		},
		version: {
			id: '3',
			version_number: '6.2.9',
			file_name: 'Entity_texture_features_fabric_1.21.1-6.2.9.jar',
		},
		owner: {
			id: '2',
			name: 'Traben',
			type: 'user',
			avatar_url: 'https://cdn.modrinth.com/user/Traben/avatar.webp',
		},
		enabled: true,
	},
	{
		project: {
			id: '4',
			slug: 'imported-mod',
			title: 'Import mod',
			icon_url: undefined,
		},
		version: {
			id: '4',
			version_number: 'Unknown',
			file_name: 'Entity_texture_features_fabric_1.21.1-6.2.9.jar',
		},
		enabled: true,
	},
])

const searchQuery = ref('')
const filterType = ref('All')
const sortType = ref('Newest')
const currentPage = ref(1)
const itemsPerPage = 10

const modpackUnlinkModal = ref<InstanceType<typeof ModpackUnlinkModal>>()

const filterOptions: ComboboxOption<string>[] = [
	{ value: 'All', label: 'All' },
	{ value: 'Mods', label: 'Mods' },
	{ value: 'Resource Packs', label: 'Resource Packs' },
	{ value: 'Shaders', label: 'Shaders' },
]

const sortOptions: ComboboxOption<string>[] = [
	{ value: 'Newest', label: 'Newest' },
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'A-Z', label: 'A-Z' },
	{ value: 'Z-A', label: 'Z-A' },
]

const filteredItems = computed(() => {
	let items = contentItems.value

	// Filter by search query
	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		items = items.filter(
			(item) =>
				item.project.title.toLowerCase().includes(query) ||
				item.owner?.name.toLowerCase().includes(query),
		)
	}

	// Sort items
	if (sortType.value === 'Oldest') {
		items = [...items].reverse()
	} else if (sortType.value === 'A-Z') {
		items = [...items].sort((a, b) => a.project.title.localeCompare(b.project.title))
	} else if (sortType.value === 'Z-A') {
		items = [...items].sort((a, b) => b.project.title.localeCompare(a.project.title))
	}

	return items
})

const totalPages = computed(() => Math.ceil(filteredItems.value.length / itemsPerPage))

const paginatedItems = computed(() => {
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredItems.value.slice(start, end)
})

const totalModsCount = computed(() => contentItems.value.length)

function goToPage(page: number) {
	currentPage.value = page
}

function handleSearch() {
	currentPage.value = 1
}

function handleToggleEnabled(item: ContentItem, value: boolean) {
	item.enabled = value
}

function handleDelete(item: ContentItem) {
	console.log('Delete:', item.project.title)
}

function handleUpdate(item: ContentItem) {
	console.log('Update:', item.project.title)
}

function handleModpackUpdate() {
	console.log('Modpack update')
}

function handleModpackContent() {
	console.log('Modpack content')
}

function handleModpackUnlink() {
	modpackUnlinkModal.value?.show()
}

function handleModpackUnlinkConfirm() {
	console.log('Modpack unlink confirmed')
}

function handleBrowseContent() {
	console.log('Browse content')
}

function handleUploadFiles() {
	console.log('Upload files')
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<ContentModpackCard
			:project="modpack.project"
			:version="modpack.version"
			:owner="modpack.owner"
			:categories="modpack.categories"
			@update="handleModpackUpdate"
			@content="handleModpackContent"
			@unlink="handleModpackUnlink"
		/>

		<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
			<div class="iconified-input flex-1 lg:max-w-lg">
				<SearchIcon aria-hidden="true" class="text-lg" />
				<input
					v-model="searchQuery"
					class="!h-10"
					autocomplete="off"
					spellcheck="false"
					type="text"
					:placeholder="`Search ${totalModsCount} mods...`"
					@input="handleSearch"
				/>
				<ButtonStyled v-if="searchQuery" circular type="transparent" class="r-btn">
					<button @click="searchQuery = ''">
						<XIcon />
					</button>
				</ButtonStyled>
			</div>

			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button class="flex items-center gap-2" @click="handleBrowseContent">
						<CompassIcon class="size-5" />
						<span>Browse content</span>
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button class="!border-surface-4 !border-[1px]" @click="handleUploadFiles">
						Upload files
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div class="flex flex-col justify-between gap-2 lg:flex-row lg:items-center">
			<div class="flex gap-2">
				<Combobox
					v-model="filterType"
					class="!w-[192px]"
					:options="filterOptions"
					@select="() => goToPage(1)"
				>
					<template #selected>
						<span class="flex items-center gap-2 font-semibold">
							<ListFilterIcon class="size-5 shrink-0 text-secondary" />
							<span class="text-contrast">{{ filterType }}</span>
						</span>
					</template>
				</Combobox>

				<Combobox
					v-model="sortType"
					class="!w-[192px]"
					:options="sortOptions"
					@select="() => goToPage(1)"
				>
					<template #selected>
						<span class="flex items-center gap-2 font-semibold">
							<SortAscIcon v-if="sortType === 'Oldest'" class="size-5 shrink-0 text-secondary" />
							<SortDescIcon v-else class="size-5 shrink-0 text-secondary" />
							<span class="text-contrast">{{ sortType }}</span>
						</span>
					</template>
				</Combobox>
			</div>

			<Pagination
				v-if="totalPages > 1"
				:page="currentPage"
				:count="totalPages"
				@switch-page="goToPage"
			/>
		</div>

		<div class="flex flex-col gap-4">
			<div
				v-if="paginatedItems.length === 0"
				class="universal-card flex h-24 items-center justify-center text-secondary"
			>
				No content found.
			</div>
			<ContentCard
				v-for="item in paginatedItems"
				:key="item.project.id"
				:project="item.project"
				:version="item.version"
				:owner="item.owner"
				:enabled="item.enabled"
				@update:enabled="(val) => handleToggleEnabled(item, val)"
				@delete="() => handleDelete(item)"
				@update="item.hasUpdate ? () => handleUpdate(item) : undefined"
			/>
		</div>

		<div v-if="totalPages > 1" class="mt-4 flex justify-center">
			<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
		</div>

		<ModpackUnlinkModal ref="modpackUnlinkModal" @unlink="handleModpackUnlinkConfirm" />
	</div>
</template>
