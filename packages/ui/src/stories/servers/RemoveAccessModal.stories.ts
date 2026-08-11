import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
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
		components: { RemoveAccessModal, Button },
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
				<Button type="colored" color="red" @click="modalRef?.show()">Remove user</Button>
				<p v-if="removed" class="m-0 text-sm text-secondary">User removed</p>
				<RemoveAccessModal
					ref="modalRef"
					username="Geometrically"
					role="editor"
					:joined-at="new Date(Date.now() - 21 * 24 * 60 * 60 * 1000).toISOString()"
					@remove="handleRemove"
				/>
			</div>
		`,
	}),
}

export const CancelInvite: Story = {
	render: () => ({
		components: { RemoveAccessModal, Button },
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
				<Button type="colored" color="red" @click="modalRef?.show()">Cancel invite</Button>
				<p v-if="cancelled" class="m-0 text-sm text-secondary">Invite cancelled</p>
				<RemoveAccessModal
					ref="modalRef"
					username="Geometrically"
					role="viewer"
					:joined-at="null"
					pending
					should-cancel
					@remove="handleCancel"
				/>
			</div>
		`,
	}),
}
