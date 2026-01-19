import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import ContentCard from '../../components/instances/ContentCard.vue'
import type {
	ContentModpackCardCategory,
	ContentModpackCardOwner,
	ContentModpackCardProject,
	ContentModpackCardVersion,
} from '../../components/instances/ContentModpackCard.vue'
import ContentModpackCard from '../../components/instances/ContentModpackCard.vue'
import NewModal from '../../components/modal/NewModal.vue'

// Real project data from Modrinth API
const fabulouslyOptimizedProject: ContentModpackCardProject = {
	id: '1KVo5zza',
	slug: 'fabulously-optimized',
	title: 'Fabulously Optimized',
	icon_url:
		'https://cdn.modrinth.com/data/1KVo5zza/9f1ded4949c2a9db5ca382d3bcc912c7245486b4_96.webp',
	description:
		'Beautiful graphics, speedy performance and familiar features in a simple package. 1.21.11 beta!',
	downloads: 8708191,
	followers: 3762,
}

const cobblemonProject: ContentModpackCardProject = {
	id: '5FFgwNNP',
	slug: 'cobblemon-fabric',
	title: 'Cobblemon Official Modpack [Fabric]',
	icon_url: 'https://cdn.modrinth.com/data/5FFgwNNP/e7f9ee2e9d361623847853fe2ddce42f519ee64f.png',
	description: 'The official modpack of the Cobblemon mod, for Fabric!',
	downloads: 4940845,
	followers: 2051,
}

const simplyOptimizedProject: ContentModpackCardProject = {
	id: 'BYfVnHa7',
	slug: 'sop',
	title: 'Simply Optimized',
	icon_url: 'https://cdn.modrinth.com/data/BYfVnHa7/845e93223da7e8d1ed1a33364b5bdb4c316ac518.png',
	description:
		'The leading, well-researched optimization modpack with a focus on pure performance.',
	downloads: 2903242,
	followers: 1387,
}

// Version data from Modrinth API
const fabulouslyOptimizedVersion: ContentModpackCardVersion = {
	id: 'YEEXo8mO',
	version_number: '1.12.1',
	date_published: '2022-02-10T06:53:28.379507Z',
}

const cobblemonVersion: ContentModpackCardVersion = {
	id: 'bpaivauC',
	version_number: '1.5.2',
	date_published: '2024-05-27T07:12:36.043005Z',
}

// Owner data from Modrinth API
const userOwner: ContentModpackCardOwner = {
	id: '2avTeeAE',
	name: 'robotkoer',
	avatar_url: 'https://cdn.modrinth.com/user/2avTeeAE/icon.png',
	type: 'user',
}

const cobblemonOwner: ContentModpackCardOwner = {
	id: 'AEFONbAM',
	name: 'Reisen',
	avatar_url:
		'https://cdn.modrinth.com/user/AEFONbAM/9e97453507a8245981d5cd825280f23be44f15ac.jpeg',
	type: 'user',
}

// Categories
const optimizationCategories: ContentModpackCardCategory[] = [
	{ name: 'Fabric' },
	{ name: 'Lightweight' },
	{ name: 'Multiplayer' },
	{ name: 'Optimization' },
]

const cobblemonCategories: ContentModpackCardCategory[] = [
	{ name: 'Adventure' },
	{ name: 'Fabric' },
	{ name: 'Lightweight' },
	{ name: 'Multiplayer' },
]

const meta = {
	title: 'Instances/ContentModpackCard',
	component: ContentModpackCard,
	parameters: {
		layout: 'padded',
	},
	argTypes: {
		project: {
			control: 'object',
			description:
				'Project information (id, slug, title, icon_url, description, downloads, followers)',
		},
		version: {
			control: 'object',
			description: 'Version information (id, version_number, date_published)',
		},
		owner: {
			control: 'object',
			description: 'Owner/author information (user or organization)',
		},
		categories: {
			control: 'object',
			description: 'Category tags with optional click actions',
		},
		disabled: {
			control: 'boolean',
			description: 'Grays out the card when true',
		},
		overflowOptions: {
			control: 'object',
			description: 'Options for the overflow menu',
		},
	},
} satisfies Meta<typeof ContentModpackCard>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// All Types Overview
// ============================================

export const AllTypes: Story = {
	args: {
		project: fabulouslyOptimizedProject,
	},
	render: () => ({
		components: { ContentModpackCard },
		setup() {
			const cards = [
				{
					label: 'Full featured (all actions)',
					project: fabulouslyOptimizedProject,
					version: fabulouslyOptimizedVersion,
					owner: userOwner,
					categories: optimizationCategories,
					hasUpdate: true,
					hasContent: true,
					hasUnlink: true,
				},
				{
					label: 'With update available only',
					project: cobblemonProject,
					version: cobblemonVersion,
					owner: cobblemonOwner,
					categories: cobblemonCategories,
					hasUpdate: true,
				},
				{
					label: 'With content button only',
					project: simplyOptimizedProject,
					version: fabulouslyOptimizedVersion,
					owner: userOwner,
					hasContent: true,
				},
				{
					label: 'Minimal (project only)',
					project: fabulouslyOptimizedProject,
				},
				{
					label: 'With version info only',
					project: cobblemonProject,
					version: cobblemonVersion,
				},
				{
					label: 'With owner only',
					project: simplyOptimizedProject,
					owner: userOwner,
				},
				{
					label: 'Disabled state',
					project: fabulouslyOptimizedProject,
					version: fabulouslyOptimizedVersion,
					owner: userOwner,
					categories: optimizationCategories,
					disabled: true,
				},
			]

			return { cards }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-6">
				<template v-for="card in cards" :key="card.label">
					<h3 class="text-sm font-medium text-secondary">{{ card.label }}</h3>
					<ContentModpackCard
						:project="card.project"
						:version="card.version"
						:owner="card.owner"
						:categories="card.categories"
						:disabled="card.disabled"
						@update="card.hasUpdate ? () => {} : undefined"
						@content="card.hasContent ? () => {} : undefined"
						@unlink="card.hasUnlink ? () => {} : undefined"
					/>
				</template>
			</div>
		`,
	}),
}

// ============================================
// Basic Stories
// ============================================

export const Default: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		onUpdate: fn(),
		onContent: fn(),
		onUnlink: fn(),
	},
}

export const MinimalProjectOnly: Story = {
	args: {
		project: cobblemonProject,
	},
}

export const WithVersion: Story = {
	args: {
		project: simplyOptimizedProject,
		version: fabulouslyOptimizedVersion,
	},
}

export const WithUserOwner: Story = {
	args: {
		project: simplyOptimizedProject,
		version: fabulouslyOptimizedVersion,
		owner: userOwner,
		categories: [{ name: 'Adventure' }],
	},
}

export const WithOrganizationOwner: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
	},
}

// ============================================
// Action Button Stories
// ============================================

export const WithUpdateButton: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		onUpdate: fn(),
	},
}

export const WithContentButton: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		onContent: fn(),
	},
}

export const WithUnlinkButton: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		onUnlink: fn(),
	},
}

export const WithAllActions: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		onUpdate: fn(),
		onContent: fn(),
		onUnlink: fn(),
		overflowOptions: [
			{ id: 'view', action: () => console.log('View') },
			{ id: 'settings', action: () => console.log('Settings') },
			{ divider: true },
			{ id: 'remove', action: () => console.log('Remove'), color: 'red' },
		],
	},
}

// ============================================
// State Stories
// ============================================

export const Disabled: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		disabled: true,
	},
}

export const LongTitle: Story = {
	args: {
		project: {
			...cobblemonProject,
			title: 'Super Long Modpack Title That Should Display Properly On All Screen Sizes',
			description:
				'This is an extremely long description that should wrap properly and not break the layout. It contains lots of information about what this modpack includes and what makes it special compared to other modpacks available on the platform.',
		},
		version: cobblemonVersion,
		owner: {
			...userOwner,
			name: 'Really Long Organization Name Studios',
		},
		categories: [
			{ name: 'Adventure' },
			{ name: 'Technology' },
			{ name: 'Magic' },
			{ name: 'Exploration' },
			{ name: 'Multiplayer' },
		],
		onUpdate: fn(),
		onContent: fn(),
	},
}

export const NoDescription: Story = {
	args: {
		project: {
			...cobblemonProject,
			description: undefined,
		},
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
	},
}

export const NoStats: Story = {
	args: {
		project: {
			...cobblemonProject,
			downloads: undefined,
			followers: undefined,
		},
		version: cobblemonVersion,
		owner: userOwner,
	},
}

// ============================================
// Categories Stories
// ============================================

export const WithClickableCategories: Story = {
	render: (args) => ({
		components: { ContentModpackCard },
		setup() {
			const clickedCategory = ref<string | null>(null)
			const categories: ContentModpackCardCategory[] = [
				{ name: 'Adventure', action: () => (clickedCategory.value = 'Adventure') },
				{ name: 'Lightweight', action: () => (clickedCategory.value = 'Lightweight') },
				{ name: 'Multiplayer', action: () => (clickedCategory.value = 'Multiplayer') },
			]
			return { args, categories, clickedCategory }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<ContentModpackCard
					:project="args.project"
					:version="args.version"
					:owner="args.owner"
					:categories="categories"
				/>
				<div class="text-sm text-secondary">
					Clicked category: <strong>{{ clickedCategory || 'None' }}</strong>
				</div>
			</div>
		`,
	}),
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
	},
}

// ============================================
// Overflow Menu Stories
// ============================================

export const WithOverflowMenu: Story = {
	render: (args) => ({
		components: { ContentModpackCard },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ContentModpackCard v-bind="args">
				<template #view>View on Modrinth</template>
				<template #settings>Settings</template>
				<template #remove>Remove modpack</template>
			</ContentModpackCard>
		`,
	}),
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
		overflowOptions: [
			{ id: 'view', action: () => console.log('View') },
			{ id: 'settings', action: () => console.log('Settings') },
			{ divider: true },
			{ id: 'remove', action: () => console.log('Remove'), color: 'red' },
		],
	},
}

// ============================================
// Interactive Stories
// ============================================

export const WithContentModal: Story = {
	args: {
		project: cobblemonProject,
	},
	render: () => ({
		components: { ContentModpackCard, NewModal, ContentCard },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const modpackContent = [
				{
					project: {
						id: '1',
						slug: 'sodium',
						title: 'Sodium',
						icon_url:
							'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
					},
					version: { id: 'v1', version_number: '0.8.2', file_name: 'sodium-fabric-0.8.2.jar' },
				},
				{
					project: {
						id: '2',
						slug: 'modmenu',
						title: 'Mod Menu',
						icon_url:
							'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
					},
					version: { id: 'v2', version_number: '16.0.0', file_name: 'modmenu-16.0.0.jar' },
				},
				{
					project: {
						id: '3',
						slug: 'fabric-api',
						title: 'Fabric API',
						icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
					},
					version: { id: 'v3', version_number: '0.141.3', file_name: 'fabric-api-0.141.3.jar' },
				},
			]

			return {
				cobblemonProject,
				cobblemonVersion,
				userOwner,
				optimizationCategories,
				modalRef,
				modpackContent,
			}
		},
		template: /*html*/ `
			<div>
				<ContentModpackCard
					:project="cobblemonProject"
					:version="cobblemonVersion"
					:owner="userOwner"
					:categories="optimizationCategories"
					@content="modalRef?.show()"
					@update="() => alert('Update clicked')"
				/>
				<NewModal ref="modalRef" header="Modpack Content">
					<div class="flex flex-col gap-4">
						<ContentCard
							v-for="item in modpackContent"
							:key="item.project.id"
							:project="item.project"
							:version="item.version"
						/>
					</div>
				</NewModal>
			</div>
		`,
	}),
}

// ============================================
// Responsive Stories
// ============================================

export const ResponsiveView: Story = {
	args: {
		project: cobblemonProject,
	},
	render: () => ({
		components: { ContentModpackCard },
		setup() {
			return {
				cobblemonProject,
				cobblemonVersion,
				userOwner,
				optimizationCategories,
			}
		},
		template: /*html*/ `
			<div class="flex flex-col gap-8">
				<div>
					<h3 class="text-sm font-medium text-secondary mb-2">Desktop (full width)</h3>
					<div class="w-full">
						<ContentModpackCard
							:project="cobblemonProject"
							:version="cobblemonVersion"
							:owner="userOwner"
							:categories="optimizationCategories"
							@update="() => {}"
							@content="() => {}"
							@unlink="() => {}"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-sm font-medium text-secondary mb-2">Mobile (&lt;640px)</h3>
					<div class="w-[360px]">
						<ContentModpackCard
							:project="cobblemonProject"
							:version="cobblemonVersion"
							:owner="userOwner"
							:categories="optimizationCategories"
							@update="() => {}"
							@content="() => {}"
						/>
					</div>
				</div>
			</div>
		`,
	}),
}

// ============================================
// Edge Cases
// ============================================

export const NoIcon: Story = {
	args: {
		project: {
			...cobblemonProject,
			icon_url: undefined,
		},
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
	},
}

export const NoOwnerAvatar: Story = {
	args: {
		project: cobblemonProject,
		version: cobblemonVersion,
		owner: {
			...userOwner,
			avatar_url: undefined,
		},
		categories: optimizationCategories,
	},
}

export const HighDownloadCounts: Story = {
	args: {
		project: {
			...cobblemonProject,
			downloads: 1234567890,
			followers: 9876543,
		},
		version: cobblemonVersion,
		owner: userOwner,
		categories: optimizationCategories,
	},
}
