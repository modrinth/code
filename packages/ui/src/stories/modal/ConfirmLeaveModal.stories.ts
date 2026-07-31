import type { StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import ConfirmLeaveModal from '../../components/modal/ConfirmLeaveModal.vue'

const meta = {
	title: 'Modal/ConfirmLeaveModal',
	component: ConfirmLeaveModal,
}

export default meta
type Story = StoryObj<typeof ConfirmLeaveModal>

export const Default: Story = {
	render: () => ({
		components: { ConfirmLeaveModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
			const result = ref<string>('')
			async function openModal() {
				result.value = ''
				const confirmed = await modalRef.value?.prompt()
				result.value = confirmed ? 'User chose to leave' : 'User chose to stay'
			}
			return { modalRef, result, openModal }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="red" @click="openModal">Trigger Leave Confirmation</Button>
				<p v-if="result" class="mt-4 text-secondary">{{ result }}</p>
				<ConfirmLeaveModal ref="modalRef" />
			</div>
		`,
	}),
}

export const CustomMessages: Story = {
	render: () => ({
		components: { ConfirmLeaveModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
			const openModal = () => modalRef.value?.prompt()
			return { modalRef, openModal }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="red" @click="openModal">Discard Draft?</Button>
				<ConfirmLeaveModal
					ref="modalRef"
					title="Discard draft?"
					header="Your draft will be lost"
					body="If you leave now, your draft will not be saved."
					stay-label="Keep editing"
					leave-label="Discard"
				/>
			</div>
		`,
	}),
}

export const WarningAdmonition: Story = {
	render: () => ({
		components: { ConfirmLeaveModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
			const openModal = () => modalRef.value?.prompt()
			return { modalRef, openModal }
		},
		template: /* html */ `
			<div>
				<Button type="colored" color="orange" @click="openModal">Open Warning Variant</Button>
				<ConfirmLeaveModal
					ref="modalRef"
					admonition-type="warning"
				/>
			</div>
		`,
	}),
}
