import type { Labrinth } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentUpdaterModal from '../../components/instances/modals/ContentUpdaterModal.vue'

// Real version data from Modrinth API - Sodium (mod)
const sodiumVersions: Labrinth.Versions.v2.Version[] = [
	{
		id: '59wygFUQ',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: true,
		name: 'Sodium 0.8.2 for Fabric 1.21.11',
		version_number: 'mc1.21.11-0.8.2-fabric',
		version_type: 'release',
		changelog:
			'This release fixes a critical bug with FRAPI, as well as allowing mods to set non-monochrome icons.\n\n## Changes\n- Fixed FRAPI compatibility issues\n- Added support for non-monochrome mod icons\n- Various performance improvements',
		date_published: '2025-12-22T20:35:06.214284Z',
		downloads: 150000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.11'],
		loaders: ['fabric', 'quilt'],
	},
	{
		id: '8jueyeK2',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: false,
		name: 'Sodium 0.8.2 for NeoForge 1.21.11',
		version_number: 'mc1.21.11-0.8.2-neoforge',
		version_type: 'release',
		changelog:
			'This release fixes a critical bug with FRAPI, as well as allowing mods to set non-monochrome icons.',
		date_published: '2025-12-22T20:34:32.101126Z',
		downloads: 80000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.11'],
		loaders: ['neoforge'],
	},
	{
		id: '2IxKzI1o',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: false,
		name: 'Sodium 0.8.1 for Fabric 1.21.11',
		version_number: 'mc1.21.11-0.8.1-fabric',
		version_type: 'release',
		changelog:
			'This release adds support for the Fabric Rendering API on 1.21.11, works around AMD driver bugs, and fixes configuration screen issues.\n\n## Bug Fixes\n- Fixed AMD driver compatibility\n- Fixed configuration screen crashes\n- Improved FRAPI support',
		date_published: '2025-12-18T03:16:30.884738Z',
		downloads: 250000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.11'],
		loaders: ['fabric', 'quilt'],
	},
	{
		id: 'MLXdfyIk',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: false,
		name: 'Sodium 0.8.0 for Fabric 1.21.11',
		version_number: 'mc1.21.11-0.8.0-fabric',
		version_type: 'beta',
		changelog:
			'This release brings many bug fixes, a brand new configuration screen, and support for Minecraft 1.21.11.\n\n## New Features\n- Completely redesigned configuration screen\n- Support for Minecraft 1.21.11\n\n## Bug Fixes\n- Fixed various rendering issues\n- Improved memory usage',
		date_published: '2025-12-09T17:11:11.360476Z',
		downloads: 180000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.11'],
		loaders: ['fabric', 'quilt'],
	},
	{
		id: 'sFfidWgd',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: false,
		name: 'Sodium 0.7.3 for Fabric 1.21.10',
		version_number: 'mc1.21.10-0.7.3-fabric',
		version_type: 'release',
		changelog: 'This release fixes a stuttering issue affecting Intel cards.',
		date_published: '2025-11-10T18:51:15.477709Z',
		downloads: 320000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.9', '1.21.10'],
		loaders: ['fabric', 'quilt'],
	},
	{
		id: '24jH02Sf',
		project_id: 'AANobbMI',
		author_id: 'TEZXhE2U',
		featured: false,
		name: 'Sodium 0.7.0 for Fabric 1.21.8',
		version_number: 'mc1.21.8-0.7.0-fabric',
		version_type: 'alpha',
		changelog:
			'Major performance optimizations with quad splitting translucency sorting, improved chunk meshing, terrain rendering enhancements, and entity/particle performance improvements.\n\n## Performance\n- 30% faster chunk rendering\n- Reduced memory allocations\n- Better CPU utilization',
		date_published: '2025-09-30T15:07:01.867787Z',
		downloads: 450000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.6', '1.21.7', '1.21.8'],
		loaders: ['fabric', 'quilt'],
	},
]

// Real version data from Modrinth API - Cobblemon modpack
const cobblemonVersions: Labrinth.Versions.v2.Version[] = [
	{
		id: 'DbQNxSJ0',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: true,
		name: 'Cobblemon Official Modpack [Fabric] 1.7.1',
		version_number: '1.7.1',
		version_type: 'release',
		changelog:
			'Updated to Cobblemon 1.7.1.\n\n## Modified Mods\n- EMF\n- ETF\n- Balm\n- FancyMenu\n- JEED\n- JEI',
		date_published: '2025-11-29T02:27:41.839520Z',
		downloads: 85000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.1'],
		loaders: ['fabric'],
	},
	{
		id: 'jMz7A3RO',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: false,
		name: 'Cobblemon Official Modpack [Fabric] 1.7',
		version_number: '1.7',
		version_type: 'release',
		changelog:
			'Updated to Cobblemon 1.7.\n\n## Changes\n- Removed Medal\n- Updated Fabric API\n- Updated JEI\n- Updated rendering mods',
		date_published: '2025-11-22T00:48:57.491974Z',
		downloads: 120000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.1'],
		loaders: ['fabric'],
	},
	{
		id: '98odLiu9',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: false,
		name: 'Cobblemon Official Modpack [Fabric] 1.6.1.4',
		version_number: '1.6.1.4',
		version_type: 'release',
		changelog: 'Updated Medal to 1.0.3 to resolve a crash issue.',
		date_published: '2025-07-01T04:32:02.692075Z',
		downloads: 95000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.1'],
		loaders: ['fabric'],
	},
	{
		id: 'ZGcN3At3',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: false,
		name: 'Cobblemon Official Modpack [Fabric] 1.6.1.1',
		version_number: '1.6.1.1',
		version_type: 'beta',
		changelog:
			'## Added Mods\n- Advanced Loot Info\n- CIT Resewn\n- EMI variants\n- Medal\n- Tips\n\n## Removed Mods\n- Architectury\n- REI',
		date_published: '2025-06-11T01:10:12.921145Z',
		downloads: 78000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.1'],
		loaders: ['fabric'],
	},
	{
		id: 'cqaC80tF',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: false,
		name: 'Cobblemon Official Modpack [Fabric] 1.6.1',
		version_number: '1.6.1',
		version_type: 'release',
		changelog: 'Updated to Cobblemon 1.6.1 release.',
		date_published: '2025-01-26T06:37:28.977532Z',
		downloads: 210000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.21.1'],
		loaders: ['fabric'],
	},
	{
		id: 'bpaivauC',
		project_id: '5FFgwNNP',
		author_id: 'AEFONbAM',
		featured: false,
		name: 'Cobblemon Official Modpack [Fabric] 1.5.2',
		version_number: '1.5.2',
		version_type: 'release',
		changelog: 'Updated to Cobblemon 1.5.2. Adjusted InvMove defaults to prevent REI conflicts.',
		date_published: '2024-05-27T07:12:36.043005Z',
		downloads: 350000,
		status: 'listed',
		files: [],
		dependencies: [],
		game_versions: ['1.20.1'],
		loaders: ['fabric'],
	},
]

const meta = {
	title: 'Instances/ContentUpdaterModal',
	component: ContentUpdaterModal,
	parameters: {
		layout: 'centered',
	},
	argTypes: {
		versions: {
			control: 'object',
			description: 'Array of versions to display',
		},
		currentGameVersion: {
			control: 'text',
			description: 'Current game version for compatibility checking',
		},
		currentLoader: {
			control: 'text',
			description: 'Current loader for compatibility checking',
		},
		currentVersionId: {
			control: 'text',
			description: 'ID of the currently installed version',
		},
		header: {
			control: 'text',
			description: 'Modal header text',
		},
	},
} satisfies Meta<typeof ContentUpdaterModal>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// Mod Example (Sodium)
// ============================================

export const ModExample: Story = {
	render: (args) => ({
		components: { ContentUpdaterModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentUpdaterModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			const handleUpdate = (version: Labrinth.Versions.v2.Version) => {
				console.log('Update to version:', version)
				alert(`Updating to ${version.name}`)
			}
			return { args, modalRef, openModal, handleUpdate }
		},
		template: /*html*/ `
			<div>
				<ButtonStyled color="brand">
					<button @click="openModal">Update Sodium</button>
				</ButtonStyled>
				<ContentUpdaterModal
					ref="modalRef"
					v-bind="args"
					@update="handleUpdate"
					@cancel="() => console.log('Cancelled')"
				/>
			</div>
		`,
	}),
	args: {
		versions: sodiumVersions,
		currentGameVersion: '1.21.11',
		currentLoader: 'fabric',
		currentVersionId: '2IxKzI1o', // 0.8.1 is current
		header: 'Update mod',
		onUpdate: fn(),
		onCancel: fn(),
	},
}

// ============================================
// Modpack Example (Cobblemon)
// ============================================

export const ModpackExample: Story = {
	render: (args) => ({
		components: { ContentUpdaterModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentUpdaterModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			const handleUpdate = (version: Labrinth.Versions.v2.Version) => {
				console.log('Update to version:', version)
				alert(`Updating to ${version.name}`)
			}
			return { args, modalRef, openModal, handleUpdate }
		},
		template: /*html*/ `
			<div>
				<ButtonStyled color="brand">
					<button @click="openModal">Update Cobblemon Modpack</button>
				</ButtonStyled>
				<ContentUpdaterModal
					ref="modalRef"
					v-bind="args"
					@update="handleUpdate"
					@cancel="() => console.log('Cancelled')"
				/>
			</div>
		`,
	}),
	args: {
		versions: cobblemonVersions,
		currentGameVersion: '1.21.1',
		currentLoader: 'fabric',
		currentVersionId: 'jMz7A3RO', // 1.7 is current
		header: 'Update modpack',
		onUpdate: fn(),
		onCancel: fn(),
	},
}

// ============================================
// With Incompatible Versions
// ============================================

export const WithIncompatibleVersions: Story = {
	render: (args) => ({
		components: { ContentUpdaterModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentUpdaterModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { args, modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<ButtonStyled color="brand">
					<button @click="openModal">Update (Shows Incompatible)</button>
				</ButtonStyled>
				<ContentUpdaterModal
					ref="modalRef"
					v-bind="args"
					@update="(v) => console.log('Update:', v)"
				/>
			</div>
		`,
	}),
	args: {
		versions: sodiumVersions,
		currentGameVersion: '1.21.10', // Older version - some versions won't be compatible
		currentLoader: 'fabric',
		currentVersionId: 'sFfidWgd',
		header: 'Update mod',
	},
}

// ============================================
// All Version Types (Release, Beta, Alpha)
// ============================================

export const AllVersionTypes: Story = {
	render: (args) => ({
		components: { ContentUpdaterModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentUpdaterModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { args, modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<ButtonStyled color="brand">
					<button @click="openModal">View All Version Types</button>
				</ButtonStyled>
				<ContentUpdaterModal
					ref="modalRef"
					v-bind="args"
					@update="(v) => console.log('Update:', v)"
				/>
			</div>
		`,
	}),
	args: {
		// Sodium has release, beta, and alpha versions
		versions: sodiumVersions,
		currentGameVersion: '1.21.11',
		currentLoader: 'fabric',
		currentVersionId: '24jH02Sf', // Alpha version is current
		header: 'Update mod',
	},
}
