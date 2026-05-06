import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import RemoveAccessModal from '../../components/servers/access/RemoveAccessModal.vue'

const meta = {
	title: 'Servers/RemoveAccessModal',
	component: RemoveAccessModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof RemoveAccessModal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ButtonStyled, RemoveAccessModal },
		setup() {
			const modalRef = ref<InstanceType<typeof RemoveAccessModal> | null>(null)
			const removed = ref(false)
			function handleRemove() {
				removed.value = true
			}
			return { modalRef, removed, handleRemove }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="red">
					<button @click="modalRef?.show()">Remove user</button>
				</ButtonStyled>
				<p v-if="removed" class="m-0 text-sm text-secondary">User removed</p>
				<RemoveAccessModal ref="modalRef" username="Geometrically" @remove="handleRemove" />
			</div>
		`,
	}),
}

export const CancelInvite: Story = {
	render: () => ({
		components: { ButtonStyled, RemoveAccessModal },
		setup() {
			const modalRef = ref<InstanceType<typeof RemoveAccessModal> | null>(null)
			const cancelled = ref(false)
			function handleCancel() {
				cancelled.value = true
			}
			return { modalRef, cancelled, handleCancel }
		},
		template: /* html */ `
			<div class="flex flex-col items-center gap-4">
				<ButtonStyled color="red">
					<button @click="modalRef?.show()">Cancel invite</button>
				</ButtonStyled>
				<p v-if="cancelled" class="m-0 text-sm text-secondary">Invite cancelled</p>
				<RemoveAccessModal
					ref="modalRef"
					username="Geometrically"
					should-cancel
					@remove="handleCancel"
				/>
			</div>
		`,
	}),
}
