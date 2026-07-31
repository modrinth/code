import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentDependencyWarningModal from '../../layouts/shared/content-tab/components/modals/ContentDependencyWarningModal.vue'
import type { ContentCardTableItem } from '../../layouts/shared/content-tab/types'

const fabricApiItem: ContentCardTableItem = {
	id: 'fabric-api',
	project: {
		id: 'P7dR8mSH',
		slug: 'fabric-api',
		title: 'Fabric API',
		icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
	},
	projectLink: '/project/fabric-api',
	version: {
		id: 'Lwa1Q6e4',
		version_number: '0.141.3+1.21.6',
		file_name: 'fabric-api-0.141.3+1.21.6.jar',
	},
	versionLink: '/project/fabric-api/version/Lwa1Q6e4',
	owner: {
		id: 'fabricmc',
		name: 'FabricMC',
		avatar_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
		type: 'organization',
		link: '/organization/fabricmc',
	},
	enabled: true,
}

const sodiumItem: ContentCardTableItem = {
	id: 'sodium',
	project: {
		id: 'AANobbMI',
		slug: 'sodium',
		title: 'Sodium',
		icon_url:
			'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
	},
	projectLink: '/project/sodium',
	version: {
		id: 'sodium-version',
		version_number: 'mc1.21.6-0.6.13-fabric',
		file_name: 'sodium-fabric-0.6.13+mc1.21.6.jar',
	},
	versionLink: '/project/sodium/version/sodium-version',
	owner: {
		id: 'jellysquid3',
		name: 'jellysquid3',
		type: 'user',
		link: '/user/jellysquid3',
	},
	enabled: true,
}

const irisItem: ContentCardTableItem = {
	id: 'iris',
	project: {
		id: 'YL57xq9U',
		slug: 'iris',
		title: 'Iris Shaders',
		icon_url: 'https://cdn.modrinth.com/data/YL57xq9U/icon.png',
	},
	projectLink: '/project/iris',
	version: {
		id: 'iris-version',
		version_number: '1.8.12+1.21.6-fabric',
		file_name: 'iris-fabric-1.8.12+mc1.21.6.jar',
	},
	versionLink: '/project/iris/version/iris-version',
	owner: {
		id: 'coderbot',
		name: 'coderbot',
		type: 'user',
		link: '/user/coderbot',
	},
	enabled: true,
}

const lithiumItem: ContentCardTableItem = {
	id: 'lithium',
	project: {
		id: 'gvQqBUqZ',
		slug: 'lithium',
		title: 'Lithium',
		icon_url:
			'https://cdn.modrinth.com/data/gvQqBUqZ/d6a1873d52b7d1c82b9a8d9b1889c9c1a29ae92d_96.webp',
	},
	projectLink: '/project/lithium',
	version: {
		id: 'lithium-version',
		version_number: 'mc1.21.6-0.16.2-fabric',
		file_name: 'lithium-fabric-0.16.2+mc1.21.6.jar',
	},
	versionLink: '/project/lithium/version/lithium-version',
	owner: {
		id: 'caffeinemc',
		name: 'CaffeineMC',
		type: 'organization',
		link: '/organization/caffeinemc',
	},
	enabled: true,
}

const continuityItem: ContentCardTableItem = {
	id: 'continuity',
	project: {
		id: '1IjD5062',
		slug: 'continuity',
		title: 'Continuity',
		icon_url: 'https://cdn.modrinth.com/data/1IjD5062/icon.png',
	},
	projectLink: '/project/continuity',
	version: {
		id: 'continuity-version',
		version_number: '3.0.1-beta.2+1.21.6',
		file_name: 'continuity-3.0.1-beta.2+1.21.6.jar',
	},
	versionLink: '/project/continuity/version/continuity-version',
	owner: {
		id: 'pepper-bell',
		name: 'Pepper_Bell',
		type: 'user',
		link: '/user/pepper-bell',
	},
	enabled: true,
}

const capeProviderItem: ContentCardTableItem = {
	id: 'cape-provider',
	project: {
		id: 'cape-provider',
		slug: 'cape-provider',
		title: 'Cape Provider',
		icon_url: null,
	},
	projectLink: '/project/cape-provider',
	version: {
		id: 'cape-provider-version',
		version_number: '5.4.2',
		file_name: 'cape-provider-5.4.2.jar',
	},
	versionLink: '/project/cape-provider/version/cape-provider-version',
	owner: {
		id: 'litetex',
		name: 'litetex',
		type: 'user',
		link: '/user/litetex',
	},
	enabled: true,
}

const meta = {
	title: 'Instances/ContentDependencyWarningModal',
	component: ContentDependencyWarningModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ContentDependencyWarningModal>

export default meta
type Story = StoryObj<typeof meta>

export const InstanceDependency: Story = {
	render: () => ({
		components: { ButtonStyled, ContentDependencyWarningModal },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDependencyWarningModal> | null>(null)
			const deleted = ref(false)
			function handleDelete() {
				deleted.value = true
			}
			return {
				modalRef,
				deleted,
				handleDelete,
				fabricApiItem,
				sodiumItem,
				irisItem,
				continuityItem,
				lithiumItem,
				capeProviderItem,
			}
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="orange">
					<button @click="modalRef?.show()">Delete dependency</button>
				</ButtonStyled>
				<p v-if="deleted" class="m-0 text-sm text-secondary">Dependency deletion confirmed</p>
				<ContentDependencyWarningModal
					ref="modalRef"
					:items="[fabricApiItem]"
					item-type="project"
					:dependents="[
						{ item: sodiumItem, dependencies: [fabricApiItem] },
						{ item: irisItem, dependencies: [fabricApiItem] },
						{ item: continuityItem, dependencies: [fabricApiItem] },
						{ item: lithiumItem, dependencies: [fabricApiItem] },
						{ item: capeProviderItem, dependencies: [fabricApiItem] },
					]"
					@delete="handleDelete"
				/>
			</div>
		`,
	}),
}

export const ServerDependency: Story = {
	render: () => ({
		components: { ButtonStyled, ContentDependencyWarningModal },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDependencyWarningModal> | null>(null)
			const deleted = ref(false)
			function handleDelete() {
				deleted.value = true
			}
			return {
				modalRef,
				deleted,
				handleDelete,
				lithiumItem,
				sodiumItem,
			}
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="orange">
					<button @click="modalRef?.show()">Delete server dependency</button>
				</ButtonStyled>
				<p v-if="deleted" class="m-0 text-sm text-secondary">Server dependency deletion confirmed</p>
				<ContentDependencyWarningModal
					ref="modalRef"
					:items="[lithiumItem]"
					item-type="project"
					:dependents="[{ item: sodiumItem, dependencies: [lithiumItem] }]"
					variant="server"
					@delete="handleDelete"
				/>
			</div>
		`,
	}),
}

export const BulkDependencies: Story = {
	render: () => ({
		components: { ButtonStyled, ContentDependencyWarningModal },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDependencyWarningModal> | null>(null)
			const deleted = ref(false)
			function handleDelete() {
				deleted.value = true
			}
			return {
				modalRef,
				deleted,
				handleDelete,
				fabricApiItem,
				lithiumItem,
				sodiumItem,
				irisItem,
				continuityItem,
				capeProviderItem,
			}
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="orange">
					<button @click="modalRef?.show()">Delete selected dependencies</button>
				</ButtonStyled>
				<p v-if="deleted" class="m-0 text-sm text-secondary">Bulk dependency deletion confirmed</p>
				<ContentDependencyWarningModal
					ref="modalRef"
					:items="[fabricApiItem, lithiumItem]"
					item-type="project"
					:dependents="[
						{ item: sodiumItem, dependencies: [fabricApiItem, lithiumItem] },
						{ item: irisItem, dependencies: [fabricApiItem] },
						{ item: continuityItem, dependencies: [fabricApiItem] },
						{ item: capeProviderItem, dependencies: [lithiumItem] },
					]"
					@delete="handleDelete"
				/>
			</div>
		`,
	}),
}
