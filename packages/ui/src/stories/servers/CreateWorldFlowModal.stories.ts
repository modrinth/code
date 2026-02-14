import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import CreateWorldFlowModal from '../../components/servers/flows/create-world-flow-modal/index.vue'

const meta = {
	title: 'Servers/CreateWorldFlowModal',
	component: CreateWorldFlowModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof CreateWorldFlowModal>

export default meta
type Story = StoryObj<typeof meta>

// ============================================
// Default — Full Flow
// ============================================

export const Default: Story = {
	render: () => ({
		components: { CreateWorldFlowModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof CreateWorldFlowModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<ButtonStyled color="brand">
					<button @click="openModal">Create World</button>
				</ButtonStyled>
				<CreateWorldFlowModal
					ref="modalRef"
					@hide="() => {}"
					@browse-modpacks="() => console.log('browse-modpacks emitted')"
					@create-world="(config) => console.log('create-world emitted', config)"
				/>
			</div>
		`,
	}),
}

// ============================================
// Events Logging
// ============================================

export const WithEventLogging: Story = {
	render: () => ({
		components: { CreateWorldFlowModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof CreateWorldFlowModal> | null>(null)
			const lastEvent = ref('')
			const openModal = () => modalRef.value?.show()

			const onBrowseModpacks = () => {
				lastEvent.value = 'browse-modpacks emitted'
			}
			const onCreateWorld = (config: any) => {
				lastEvent.value = `create-world emitted — name: "${config.worldName.value}", mode: ${config.gamemode.value}`
			}
			const onHide = () => {
				lastEvent.value = 'hide emitted'
			}
			return { modalRef, openModal, lastEvent, onBrowseModpacks, onCreateWorld, onHide }
		},
		template: /*html*/ `
			<div class="flex flex-col gap-4 items-center">
				<ButtonStyled color="brand">
					<button @click="openModal">Create World (Event Logging)</button>
				</ButtonStyled>
				<p v-if="lastEvent" class="text-sm text-secondary mt-2">Last event: {{ lastEvent }}</p>
				<CreateWorldFlowModal
					ref="modalRef"
					@hide="onHide"
					@browse-modpacks="onBrowseModpacks"
					@create-world="onCreateWorld"
				/>
			</div>
		`,
	}),
}
