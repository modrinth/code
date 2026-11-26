import { Button } from '@modrinth/ui'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ModerateByIdsModal from '~/components/ui/moderation/ModerateByIdsModal.vue'

const meta = {
	title: 'Website/Moderation/ModerateByIdsModal',
	component: ModerateByIdsModal,
} satisfies Meta<typeof ModerateByIdsModal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ModerateByIdsModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ModerateByIdsModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Modal</Button>
				<ModerateByIdsModal ref="modalRef"/>
			</div>
		`,
	}),
}
