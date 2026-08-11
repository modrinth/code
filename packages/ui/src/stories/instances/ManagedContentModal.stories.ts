import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import ManagedContentModal from '../../layouts/shared/content-tab/components/managed-content-modal/index.vue'
import type { ContentItem } from '../../layouts/shared/content-tab/types'

const sodiumItem: ContentItem = {
	file_name: 'sodium-fabric-0.8.2+mc1.21.1.jar',
	file_path: '',
	hash: '',
	size: 1024000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'AANobbMI',
		slug: 'sodium',
		title: 'Sodium',
		icon_url:
			'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
	},
	version: {
		id: '59wygFUQ',
		version_number: 'mc1.21.1-0.8.2-fabric',
		file_name: 'sodium-fabric-0.8.2+mc1.21.1.jar',
	},
	owner: {
		id: 'DzLrfrbK',
		name: 'IMS',
		avatar_url: 'https://avatars3.githubusercontent.com/u/31803019?v=4',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const lithiumItem: ContentItem = {
	file_name: 'lithium-fabric-0.14.3+mc1.21.1.jar',
	file_path: '',
	hash: '',
	size: 512000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'gvQqBUqZ',
		slug: 'lithium',
		title: 'Lithium',
		icon_url:
			'https://cdn.modrinth.com/data/gvQqBUqZ/d6a1873d52b7d1c82b9a8d9b1889c9c1a29ae92d_96.webp',
	},
	version: {
		id: 'abc123',
		version_number: 'mc1.21.1-0.14.3',
		file_name: 'lithium-fabric-0.14.3+mc1.21.1.jar',
	},
	owner: {
		id: 'DzLrfrbK',
		name: 'IMS',
		avatar_url: 'https://avatars3.githubusercontent.com/u/31803019?v=4',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const fabricApiItem: ContentItem = {
	file_name: 'fabric-api-0.141.3+26.1.jar',
	file_path: '',
	hash: '',
	size: 2048000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'P7dR8mSH',
		slug: 'fabric-api',
		title: 'Fabric API',
		icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
	},
	version: {
		id: 'Lwa1Q6e4',
		version_number: '0.141.3+26.1',
		file_name: 'fabric-api-0.141.3+26.1.jar',
	},
	owner: {
		id: 'BZoBsPo6',
		name: 'FabricMC',
		avatar_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
		type: 'organization',
	},
	has_update: false,
	update_version_id: null,
}

const modMenuItem: ContentItem = {
	file_name: 'modmenu-16.0.0.jar',
	file_path: '',
	hash: '',
	size: 256000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'mOgUt4GM',
		slug: 'modmenu',
		title: 'Mod Menu',
		icon_url: 'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
	},
	version: {
		id: 'QuU0ciaR',
		version_number: '16.0.0',
		file_name: 'modmenu-16.0.0.jar',
	},
	owner: {
		id: 'u2',
		name: 'Prospector',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const irisItem: ContentItem = {
	file_name: 'iris-1.8.0+mc1.21.1.jar',
	file_path: '',
	hash: '',
	size: 1536000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'YL57xq9U',
		slug: 'iris',
		title: 'Iris Shaders',
		icon_url: 'https://cdn.modrinth.com/data/YL57xq9U/icon.png',
	},
	version: {
		id: 'iris123',
		version_number: '1.8.0+mc1.21.1',
		file_name: 'iris-1.8.0+mc1.21.1.jar',
	},
	owner: {
		id: 'coderbot',
		name: 'coderbot',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const entityModelFeaturesItem: ContentItem = {
	file_name: 'entity-model-features-fabric-2.4.1+mc1.21.1.jar',
	file_path: '',
	hash: '',
	size: 768000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'emf123',
		slug: 'entity-model-features',
		title: '[EMF] Entity Model Features',
		icon_url:
			'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
	},
	version: {
		id: 'emfv1',
		version_number: '2.4.1',
		file_name: 'Entity_model_features_fabric_1.21.1-2.4.1.jar',
	},
	owner: {
		id: 'traben',
		name: 'Traben',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const entityTextureFeaturesItem: ContentItem = {
	file_name: 'entity-texture-features-fabric-6.2.9+mc1.21.1.jar',
	file_path: '',
	hash: '',
	size: 640000,
	enabled: true,
	project_type: 'mod',
	project: {
		id: 'etf456',
		slug: 'entity-texture-features',
		title: '[ETF] Entity Texture Features',
		icon_url: 'https://cdn.modrinth.com/data/mOgUt4GM/5a20ed1450a0e1e79a1fe04e61bb4e5878bf1d20.png',
	},
	version: {
		id: 'etfv1',
		version_number: '6.2.9',
		file_name: 'Entity_texture_features_fabric_1.21.1-6.2.9.jar',
	},
	owner: {
		id: 'traben',
		name: 'Traben',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const complementaryShaderItem: ContentItem = {
	file_name: 'ComplementaryReimagined_r5.3.zip',
	file_path: '',
	hash: '',
	size: 2048000,
	enabled: true,
	project_type: 'shader',
	project: {
		id: 'shader1',
		slug: 'complementary-reimagined',
		title: 'Complementary Reimagined',
		icon_url: 'https://cdn.modrinth.com/data/HVnmMxH1/icon.png',
	},
	version: {
		id: 'shaderv1',
		version_number: 'r5.3',
		file_name: 'ComplementaryReimagined_r5.3.zip',
	},
	owner: {
		id: 'emin',
		name: 'EminGT',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const bslShaderItem: ContentItem = {
	file_name: 'BSL_v8.2.09.zip',
	file_path: '',
	hash: '',
	size: 1024000,
	enabled: true,
	project_type: 'shader',
	project: {
		id: 'shader2',
		slug: 'bsl-shaders',
		title: 'BSL Shaders',
		icon_url: 'https://cdn.modrinth.com/data/Q1vvjJYV/icon.png',
	},
	version: {
		id: 'shaderv2',
		version_number: 'v8.2.09',
		file_name: 'BSL_v8.2.09.zip',
	},
	owner: {
		id: 'capt',
		name: 'CaptTatsu',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const faithfulItem: ContentItem = {
	file_name: 'Faithful 32x - 1.21.zip',
	file_path: '',
	hash: '',
	size: 8192000,
	enabled: true,
	project_type: 'resourcepack',
	project: {
		id: 'rp1',
		slug: 'faithful-32x',
		title: 'Faithful 32x',
		icon_url: 'https://cdn.modrinth.com/data/tAnpCviC/icon.png',
	},
	version: {
		id: 'rpv1',
		version_number: '1.21',
		file_name: 'Faithful 32x - 1.21.zip',
	},
	owner: {
		id: 'faithful',
		name: 'Faithful Resource Pack',
		avatar_url: 'https://cdn.modrinth.com/data/tAnpCviC/icon.png',
		type: 'organization',
	},
	has_update: false,
	update_version_id: null,
}

const vanillaTweaksItem: ContentItem = {
	file_name: 'VanillaTweaks_r3.zip',
	file_path: '',
	hash: '',
	size: 512000,
	enabled: true,
	project_type: 'resourcepack',
	project: {
		id: 'rp2',
		slug: 'vanilla-tweaks',
		title: 'Vanilla Tweaks',
		icon_url: null,
	},
	version: {
		id: 'rpv2',
		version_number: 'r3',
		file_name: 'VanillaTweaks_r3.zip',
	},
	owner: {
		id: 'xisuma',
		name: 'Xisumavoid',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const stayTrueItem: ContentItem = {
	file_name: 'Stay_True_1.21.zip',
	file_path: '',
	hash: '',
	size: 4096000,
	enabled: true,
	project_type: 'resourcepack',
	project: {
		id: 'rp3',
		slug: 'stay-true',
		title: 'Stay True',
		icon_url: 'https://cdn.modrinth.com/data/HVnmMxH1/icon.png',
	},
	version: {
		id: 'rpv3',
		version_number: '1.21',
		file_name: 'Stay_True_1.21.zip',
	},
	owner: {
		id: 'hallowed',
		name: 'HallowedST',
		type: 'user',
	},
	has_update: false,
	update_version_id: null,
}

const mixedModpackContent: ContentItem[] = [
	sodiumItem,
	lithiumItem,
	fabricApiItem,
	modMenuItem,
	irisItem,
	entityModelFeaturesItem,
	entityTextureFeaturesItem,
	complementaryShaderItem,
	bslShaderItem,
	faithfulItem,
	vanillaTweaksItem,
	stayTrueItem,
]

const modsOnlyContent: ContentItem[] = [
	sodiumItem,
	lithiumItem,
	fabricApiItem,
	modMenuItem,
	irisItem,
	entityModelFeaturesItem,
	entityTextureFeaturesItem,
]

const largeModpackContent: ContentItem[] = [
	...mixedModpackContent,
	...Array.from({ length: 35 }, (_, i) => ({
		...sodiumItem,
		file_name: `mod-${i + 1}-1.0.0.jar`,
		project: {
			id: `mod-${i + 1}`,
			slug: `mod-${i + 1}`,
			title: `Example Mod ${i + 1}`,
			icon_url:
				i % 3 === 0
					? sodiumItem.project!.icon_url
					: i % 3 === 1
						? fabricApiItem.project!.icon_url
						: modMenuItem.project!.icon_url,
		},
		version: {
			id: `v${i + 1}`,
			version_number: `1.${i}.0`,
			file_name: `mod-${i + 1}-1.0.0.jar`,
		},
		owner: i % 2 === 0 ? sodiumItem.owner : fabricApiItem.owner,
	})),
]

const meta = {
	title: 'Instances/ManagedContentModal',
	component: ManagedContentModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ManagedContentModal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show(mixedModpackContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<Button type="colored" color="brand" @click="openModal">View Modpack Content (Mixed)</Button>
				<ManagedContentModal
					ref="modalRef"
					source-name="Cobblemon Official Modpack"
					source-icon-url="https://cdn.modrinth.com/data/5FFgwNNP/icon.png"
				/>
			</div>
		`,
	}),
}

export const ModsOnly: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show(modsOnlyContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<Button type="colored" color="brand" @click="openModal">View Modpack Content (Mods Only)</Button>
				<ManagedContentModal ref="modalRef" />
			</div>
		`,
	}),
}

export const LoadingState: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => {
				modalRef.value?.showLoading()
				setTimeout(() => {
					modalRef.value?.show(mixedModpackContent)
				}, 2000)
			}
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<Button type="colored" color="brand" @click="openModal">View Content (With Loading)</Button>
				<ManagedContentModal
					ref="modalRef"
					source-name="Fabulously Optimized"
					source-icon-url="https://cdn.modrinth.com/data/1KVo5zza/icon.png"
				/>
			</div>
		`,
	}),
}

export const EmptyContent: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show([])
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<Button type="colored" color="brand" @click="openModal">View Empty Modpack</Button>
				<ManagedContentModal ref="modalRef" />
			</div>
		`,
	}),
}

export const LargeModpack: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show(largeModpackContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div>
				<Button type="colored" color="brand" @click="openModal">View Large Modpack (47 items)</Button>
				<ManagedContentModal
					ref="modalRef"
					source-name="All the Mods 10"
					source-icon-url="https://cdn.modrinth.com/data/1KVo5zza/icon.png"
				/>
			</div>
		`,
	}),
}

export const SearchDemo: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show(mixedModpackContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<p class="text-sm text-secondary max-w-md text-center">
					Click the button and try searching for "sodium", "shader", or "faithful" to test the search functionality.
				</p>
				<Button type="colored" color="brand" @click="openModal">Test Search</Button>
				<ManagedContentModal ref="modalRef" />
			</div>
		`,
	}),
}

export const FilterDemo: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const openModal = () => modalRef.value?.show(mixedModpackContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<p class="text-sm text-secondary max-w-md text-center">
					Click the button and try the filter chips (Mods, Shaders, Resource Packs) to filter content by type.
				</p>
				<Button type="colored" color="brand" @click="openModal">Test Filters</Button>
				<ManagedContentModal ref="modalRef" />
			</div>
		`,
	}),
}

export const MixedOwnerTypes: Story = {
	render: () => ({
		components: { ManagedContentModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ManagedContentModal> | null>(null)
			const mixedContent = [
				sodiumItem, // User owner
				fabricApiItem, // Organization owner
				modMenuItem, // User owner without avatar
				faithfulItem, // Organization owner
			]
			const openModal = () => modalRef.value?.show(mixedContent)
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<p class="text-sm text-secondary max-w-md text-center">
					Shows content with different owner types: users (circular avatar) and organizations (rounded + icon).
				</p>
				<Button type="colored" color="brand" @click="openModal">View Mixed Owners</Button>
				<ManagedContentModal ref="modalRef" />
			</div>
		`,
	}),
}
