import { EditIcon, EyeIcon, FolderOpenIcon, LinkIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import type {
	ContentCardOwner,
	ContentCardProject,
	ContentCardVersion,
} from '../../components/instances/ContentCard.vue'
import ContentCard from '../../components/instances/ContentCard.vue'

// Real project data from Modrinth API
const sodiumProject: ContentCardProject = {
	id: 'AANobbMI',
	slug: 'sodium',
	title: 'Sodium',
	icon_url:
		'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
}

const modMenuProject: ContentCardProject = {
	id: 'mOgUt4GM',
	slug: 'modmenu',
	title: 'Mod Menu',
	icon_url: 'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
}

const fabricApiProject: ContentCardProject = {
	id: 'P7dR8mSH',
	slug: 'fabric-api',
	title: 'Fabric API',
	icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
}

// Version data
const sodiumVersion: ContentCardVersion = {
	id: '59wygFUQ',
	version_number: 'mc1.21.11-0.8.2-fabric',
	file_name: 'sodium-fabric-0.8.2+mc1.21.11.jar',
}

const modMenuVersion: ContentCardVersion = {
	id: 'QuU0ciaR',
	version_number: '16.0.0',
	file_name: 'modmenu-16.0.0.jar',
}

const fabricApiVersion: ContentCardVersion = {
	id: 'Lwa1Q6e4',
	version_number: '0.141.3+26.1',
	file_name: 'fabric-api-0.141.3+26.1.jar',
}

// Owner data
const sodiumOwner: ContentCardOwner = {
	id: 'DzLrfrbK',
	name: 'IMS',
	avatar_url: 'https://avatars3.githubusercontent.com/u/31803019?v=4',
	type: 'user',
}

const fabricApiOwner: ContentCardOwner = {
	id: 'BZoBsPo6',
	name: 'FabricMC',
	avatar_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
	type: 'organization',
}

const meta = {
	title: 'Instances/ContentCard',
	component: ContentCard,
	parameters: {
		layout: 'padded',
	},
	argTypes: {
		project: {
			control: 'object',
			description: 'Project information (id, slug, title, icon_url)',
		},
		version: {
			control: 'object',
			description: 'Version information (id, version_number, file_name)',
		},
		owner: {
			control: 'object',
			description: 'Owner/author information',
		},
		enabled: {
			control: 'boolean',
			description: 'Toggle state - toggle hidden if undefined',
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
} satisfies Meta<typeof ContentCard>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// All Types Overview
// ============================================

export const AllTypes: Story = {
	args: {
		project: sodiumProject,
	},
	render: () => ({
		components: { ContentCard },
		setup() {
			const toggleOn = ref(true)
			const toggleOff = ref(false)

			const cards = [
				{
					label: 'Full featured (all actions)',
					project: sodiumProject,
					version: sodiumVersion,
					owner: sodiumOwner,
					enabled: toggleOn,
					hasUpdate: true,
					hasDelete: true,
					hasOverflow: true,
				},
				{
					label: 'With toggle only',
					project: modMenuProject,
					version: modMenuVersion,
					owner: { id: 'u2', name: 'Prospector', type: 'user' },
					enabled: toggleOn,
				},
				{
					label: 'With update available',
					project: fabricApiProject,
					version: fabricApiVersion,
					owner: fabricApiOwner,
					hasUpdate: true,
				},
				{
					label: 'Minimal (project only)',
					project: sodiumProject,
				},
				{
					label: 'With version info only',
					project: modMenuProject,
					version: modMenuVersion,
				},
				{
					label: 'With owner only',
					project: fabricApiProject,
					owner: fabricApiOwner,
				},
				{
					label: 'Disabled state',
					project: sodiumProject,
					version: sodiumVersion,
					owner: sodiumOwner,
					enabled: toggleOff,
					disabled: true,
				},
				{
					label: 'Delete button only',
					project: modMenuProject,
					version: modMenuVersion,
					hasDelete: true,
				},
				{
					label: 'Toggle off',
					project: fabricApiProject,
					version: fabricApiVersion,
					owner: fabricApiOwner,
					enabled: toggleOff,
				},
			]

			return { cards }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<template v-for="card in cards" :key="card.label">
					<h3 class="text-sm font-medium text-secondary">{{ card.label }}</h3>
					<ContentCard
						:project="card.project"
						:version="card.version"
						:owner="card.owner"
						:enabled="card.enabled?.value"
						:disabled="card.disabled"
						@update="card.hasUpdate ? () => {} : undefined"
						@delete="card.hasDelete ? () => {} : undefined"
						@update:enabled="card.enabled ? (v) => card.enabled.value = v : undefined"
						:overflow-options="card.hasOverflow ? [
							{ id: 'view', action: () => {} },
							{ divider: true },
							{ id: 'remove', action: () => {}, color: 'red' },
						] : undefined"
					>
						<template #view>View on Modrinth</template>
						<template #remove>Remove</template>
					</ContentCard>
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
		project: sodiumProject,
		version: sodiumVersion,
		owner: sodiumOwner,
		enabled: true,
		overflowOptions: [
			{ id: 'view', action: () => console.log('View clicked') },
			{ id: 'edit', action: () => console.log('Edit clicked') },
			{ divider: true },
			{ id: 'remove', action: () => console.log('Remove clicked'), color: 'red' },
		],
		onDelete: fn(),
		onUpdate: fn(),
		'onUpdate:enabled': fn(),
	},
}

export const MinimalProjectOnly: Story = {
	args: {
		project: sodiumProject,
	},
}

export const WithVersion: Story = {
	args: {
		project: modMenuProject,
		version: modMenuVersion,
	},
}

export const WithOwner: Story = {
	args: {
		project: fabricApiProject,
		owner: fabricApiOwner,
	},
}

export const WithToggle: Story = {
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		enabled: true,
		'onUpdate:enabled': fn(),
	},
}

export const ToggleDisabled: Story = {
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		enabled: false,
		'onUpdate:enabled': fn(),
	},
}

// ============================================
// Action Button Stories
// ============================================

export const WithDeleteButton: Story = {
	args: {
		project: modMenuProject,
		version: modMenuVersion,
		owner: sodiumOwner,
		onDelete: fn(),
	},
}

export const WithUpdateButton: Story = {
	args: {
		project: fabricApiProject,
		version: fabricApiVersion,
		owner: fabricApiOwner,
		onUpdate: fn(),
	},
}

export const WithAllActions: Story = {
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		owner: sodiumOwner,
		enabled: true,
		onDelete: fn(),
		onUpdate: fn(),
		'onUpdate:enabled': fn(),
		overflowOptions: [
			{ id: 'view', action: () => console.log('View') },
			{ id: 'openFolder', action: () => console.log('Open folder') },
			{ divider: true },
			{ id: 'copyLink', action: () => console.log('Copy link') },
		],
	},
}

// ============================================
// State Stories
// ============================================

export const Disabled: Story = {
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		owner: sodiumOwner,
		enabled: false,
		disabled: true,
		'onUpdate:enabled': fn(),
	},
}

export const LongProjectName: Story = {
	args: {
		project: {
			id: 'test123',
			slug: 'very-long-project-name',
			title: '[EMF] Entity Model Features - The Ultimate Entity Rendering Mod',
			icon_url: sodiumProject.icon_url,
		},
		version: {
			id: 'v1',
			version_number: '2.4.1',
			file_name: 'Entity_model_features_fabric_1.21.1-2.4.1.jar',
		},
		owner: {
			id: 'u1',
			name: 'Traben',
			type: 'user',
		},
		enabled: true,
		onDelete: fn(),
		'onUpdate:enabled': fn(),
	},
}

// ============================================
// Overflow Menu Stories
// ============================================

export const WithOverflowMenu: Story = {
	render: (args) => ({
		components: { ContentCard, EditIcon, EyeIcon, FolderOpenIcon, LinkIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ContentCard v-bind="args">
				<template #view>
					<EyeIcon class="size-5" /> View on Modrinth
				</template>
				<template #edit>
					<EditIcon class="size-5" /> Edit settings
				</template>
				<template #openFolder>
					<FolderOpenIcon class="size-5" /> Open folder
				</template>
				<template #copyLink>
					<LinkIcon class="size-5" /> Copy link
				</template>
			</ContentCard>
		`,
	}),
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		owner: sodiumOwner,
		overflowOptions: [
			{ id: 'view', action: () => console.log('View') },
			{ id: 'edit', action: () => console.log('Edit') },
			{ id: 'openFolder', action: () => console.log('Open folder') },
			{ divider: true },
			{ id: 'copyLink', action: () => console.log('Copy link') },
		],
	},
}

// ============================================
// Slot Stories
// ============================================

export const WithAdditionalButtons: Story = {
	render: (args) => ({
		components: { ContentCard, ButtonStyled, EyeIcon, FolderOpenIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ContentCard v-bind="args">
				<template #additionalButtonsLeft>
					<ButtonStyled v-tooltip="'View on Modrinth'" circular type="transparent">
						<button>
							<EyeIcon class="size-5 text-secondary" />
						</button>
					</ButtonStyled>
				</template>
				<template #additionalButtonsRight>
					<ButtonStyled v-tooltip="'Open folder'" circular type="transparent">
						<button>
							<FolderOpenIcon class="size-5 text-secondary" />
						</button>
					</ButtonStyled>
				</template>
			</ContentCard>
		`,
	}),
	args: {
		project: modMenuProject,
		version: modMenuVersion,
		enabled: true,
		onDelete: fn(),
		'onUpdate:enabled': fn(),
	},
}

// ============================================
// Interactive Stories
// ============================================

export const InteractiveToggle: Story = {
	args: {
		project: sodiumProject,
	},
	render: () => ({
		components: { ContentCard },
		setup() {
			const enabled = ref(true)
			return {
				enabled,
				project: sodiumProject,
				version: sodiumVersion,
				owner: sodiumOwner,
			}
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<ContentCard
					:project="project"
					:version="version"
					:owner="owner"
					v-model:enabled="enabled"
				/>
				<div class="text-sm text-secondary">
					Mod is currently: <strong>{{ enabled ? 'Enabled' : 'Disabled' }}</strong>
				</div>
			</div>
		`,
	}),
}

// ============================================
// List Stories
// ============================================

export const ModList: Story = {
	args: {
		project: sodiumProject,
	},
	render: () => ({
		components: { ContentCard },
		setup() {
			const mods = ref([
				{ project: sodiumProject, version: sodiumVersion, owner: sodiumOwner, enabled: true },
				{
					project: modMenuProject,
					version: modMenuVersion,
					owner: { id: 'u2', name: 'Prospector', type: 'user' as const },
					enabled: true,
				},
				{
					project: fabricApiProject,
					version: fabricApiVersion,
					owner: fabricApiOwner,
					enabled: true,
				},
			])

			const handleDelete = (index: number) => {
				mods.value.splice(index, 1)
			}

			const handleToggle = (index: number, value: boolean) => {
				mods.value[index].enabled = value
			}

			return { mods, handleDelete, handleToggle }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-3">
				<ContentCard
					v-for="(mod, index) in mods"
					:key="mod.project.id"
					:project="mod.project"
					:version="mod.version"
					:owner="mod.owner"
					:enabled="mod.enabled"
					@update:enabled="(val) => handleToggle(index, val)"
					@delete="() => handleDelete(index)"
					:overflow-options="[
						{ id: 'view', action: () => console.log('View', mod.project.slug) },
						{ divider: true },
						{ id: 'remove', action: () => handleDelete(index), color: 'red' },
					]"
				>
					<template #view>View on Modrinth</template>
					<template #remove>Remove from instance</template>
				</ContentCard>
			</div>
		`,
	}),
}

export const MixedStates: Story = {
	args: {
		project: sodiumProject,
	},
	render: () => ({
		components: { ContentCard },
		setup() {
			return {
				sodiumProject,
				sodiumVersion,
				sodiumOwner,
				modMenuProject,
				modMenuVersion,
				fabricApiProject,
				fabricApiVersion,
				fabricApiOwner,
			}
		},
		template: /*html*/ `
			<div class="flex flex-col gap-3">
				<!-- Enabled with update available -->
				<ContentCard
					:project="sodiumProject"
					:version="sodiumVersion"
					:owner="sodiumOwner"
					:enabled="true"
					@update="() => console.log('Update sodium')"
					@update:enabled="() => {}"
					@delete="() => {}"
				/>

				<!-- Disabled mod -->
				<ContentCard
					:project="modMenuProject"
					:version="modMenuVersion"
					:enabled="false"
					:disabled="true"
					@update:enabled="() => {}"
				/>

				<!-- No toggle, just view -->
				<ContentCard
					:project="fabricApiProject"
					:version="fabricApiVersion"
					:owner="fabricApiOwner"
				/>
			</div>
		`,
	}),
}

// ============================================
// Responsive Stories
// ============================================

export const ResponsiveView: Story = {
	args: {
		project: sodiumProject,
	},
	render: () => ({
		components: { ContentCard },
		setup() {
			return {
				project: sodiumProject,
				version: sodiumVersion,
				owner: sodiumOwner,
			}
		},
		template: /*html*/ `
			<div class="flex flex-col gap-6">
				<div>
					<h3 class="text-sm font-medium text-secondary mb-2">Desktop (version info visible)</h3>
					<div class="w-full">
						<ContentCard
							:project="project"
							:version="version"
							:owner="owner"
							:enabled="true"
							@update:enabled="() => {}"
							@delete="() => {}"
							@update="() => {}"
						/>
					</div>
				</div>
				<div>
					<h3 class="text-sm font-medium text-secondary mb-2">Mobile (&lt;768px - version info hidden)</h3>
					<div class="w-[360px]">
						<ContentCard
							:project="project"
							:version="version"
							:owner="owner"
							:enabled="true"
							@update:enabled="() => {}"
							@delete="() => {}"
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
			id: 'test',
			slug: 'no-icon-mod',
			title: 'Mod Without Icon',
			icon_url: undefined,
		},
		version: {
			id: 'v1',
			version_number: '1.0.0',
			file_name: 'no-icon-mod-1.0.0.jar',
		},
		enabled: true,
		'onUpdate:enabled': fn(),
	},
}

export const NoOwnerAvatar: Story = {
	args: {
		project: sodiumProject,
		version: sodiumVersion,
		owner: {
			id: 'u1',
			name: 'Anonymous',
			avatar_url: undefined,
			type: 'user',
		},
		enabled: true,
		'onUpdate:enabled': fn(),
	},
}
