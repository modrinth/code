import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import type {
	CreationFlowContextValue,
	ProjectInstallCreateData,
} from '../../components/flows/creation-flow-modal/creation-flow-context'
import CreationFlowModal from '../../components/flows/creation-flow-modal/index.vue'

const meta = {
	title: 'Servers/CreationFlowModal',
	component: CreationFlowModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof CreationFlowModal>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// Create World (Hosting)
// ============================================

export const CreateWorld: Story = {
	name: 'Create World (Hosting)',
	render: () => ({
		components: { CreationFlowModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
			const lastEvent = ref('')
			const openModal = () => modalRef.value?.show()

			const onCreate = (config: CreationFlowContextValue) => {
				lastEvent.value = `create emitted — name: "${config.worldName.value}", mode: ${config.gamemode.value}`
			}
			return { modalRef, openModal, lastEvent, onCreate }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<Button type="colored" color="brand" @click="openModal">Create World</Button>
				<p v-if="lastEvent" class="text-sm text-secondary mt-2">Last event: {{ lastEvent }}</p>
				<CreationFlowModal
					ref="modalRef"
					type="world"
					:show-snapshot-toggle="true"
					@hide="() => {}"
					@browse-modpacks="() => console.log('browse-modpacks emitted')"
					@create="onCreate"
				/>
			</div>
		`,
	}),
}

// ============================================
// Server Setup (Legacy) (Hosting)
// ============================================

export const ServerOnboarding: Story = {
	name: 'Server Setup (Legacy) (Hosting)',
	render: () => ({
		components: { CreationFlowModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
			const lastEvent = ref('')
			const openModal = () => modalRef.value?.show()

			const onCreate = (config: CreationFlowContextValue) => {
				lastEvent.value = `create emitted — loader: ${config.selectedLoader.value}, version: ${config.selectedGameVersion.value}`
			}
			return { modalRef, openModal, lastEvent, onCreate }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<Button type="colored" color="brand" @click="openModal">Set Up Server</Button>
				<p v-if="lastEvent" class="text-sm text-secondary mt-2">Last event: {{ lastEvent }}</p>
				<CreationFlowModal
					ref="modalRef"
					type="server-onboarding"
					:show-snapshot-toggle="true"
					@hide="() => {}"
					@browse-modpacks="() => console.log('browse-modpacks emitted')"
					@create="onCreate"
				/>
			</div>
		`,
	}),
}

// ============================================
// Create Instance (App)
// ============================================

export const Instance: Story = {
	name: 'Create Instance (App)',
	render: () => ({
		components: { CreationFlowModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
			const lastEvent = ref('')
			const openModal = () => modalRef.value?.show()

			const onCreate = (config: CreationFlowContextValue) => {
				lastEvent.value = `create emitted — loader: ${config.selectedLoader.value}, version: ${config.selectedGameVersion.value}`
			}
			const prepareProjectInstall = async (projectId: string, projectType: string) => {
				lastEvent.value = `project selected — ${projectId}`
				if (projectType === 'modpack') return null
				return {
					projectId,
					title: 'Fabric API',
					iconUrl: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
					link: '/project/fabric-api',
					compatibleLoaders: ['fabric'],
					gameVersions: ['1.21.5', '1.21.4'],
					releaseGameVersions: new Set(['1.21.5', '1.21.4']),
				}
			}
			const createProjectInstall = async (data: ProjectInstallCreateData) => {
				lastEvent.value = `project instance created — ${data.name}`
			}
			const searchProjects = async () => ({
				hits: [
					{
						project_id: 'fabric-api',
						project_type: 'mod',
						title: 'Fabric API',
						icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
					},
					{
						project_id: 'adrenaline',
						project_type: 'modpack',
						title: 'Adrenaline',
						icon_url: 'https://cdn.modrinth.com/data/RPYx1T0D/icon.png',
					},
				],
				offset: 0,
				limit: 10,
				total_hits: 2,
			})
			return {
				modalRef,
				openModal,
				lastEvent,
				onCreate,
				prepareProjectInstall,
				createProjectInstall,
				searchProjects,
			}
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<Button type="colored" color="brand" @click="openModal">Create Instance</Button>
				<p v-if="lastEvent" class="text-sm text-secondary mt-2">Last event: {{ lastEvent }}</p>
				<CreationFlowModal
					ref="modalRef"
					type="instance"
					:show-snapshot-toggle="true"
					:search-projects="searchProjects"
					:prepare-project-install="prepareProjectInstall"
					:create-project-install="createProjectInstall"
					@hide="() => {}"
					@browse-modpacks="() => console.log('browse-modpacks emitted')"
					@create="onCreate"
				/>
			</div>
		`,
	}),
}
