import type { StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import NewModal from '../../components/modal/NewModal.vue'

const meta = {
	title: 'Modal/NewModal',
	component: NewModal,
}

export default meta
type Story = StoryObj<typeof NewModal>

export const Default: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Modal</Button>
				<NewModal ref="modalRef" header="Example Modal">
					<p>This is the modal content.</p>
					<p class="text-secondary mt-2">You can put any content here.</p>
				</NewModal>
			</div>
		`,
	}),
}

export const WithActions: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Modal with Actions</Button>
				<NewModal ref="modalRef" header="Confirm Action">
					<p>Are you sure you want to proceed with this action?</p>
					<template #actions>
						<div class="flex gap-2 justify-end">
							<Button @click="modalRef?.hide()">Cancel</Button>
							<Button type="colored" color="brand" @click="modalRef?.hide()">Confirm</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}

export const DangerFade: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="red" @click="openModal">Open Danger Modal</Button>
				<NewModal ref="modalRef" header="Delete Item" fade="danger">
					<p>Are you sure you want to delete this item? This action cannot be undone.</p>
					<template #actions>
						<div class="flex gap-2 justify-end">
							<Button @click="modalRef?.hide()">Cancel</Button>
							<Button type="colored" color="red" @click="modalRef?.hide()">Delete</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}

export const WarningFade: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="orange" @click="openModal">Open Warning Modal</Button>
				<NewModal ref="modalRef" header="Warning" fade="warning">
					<p>This action may have unintended consequences. Please review before proceeding.</p>
					<template #actions>
						<div class="flex gap-2 justify-end">
							<Button @click="modalRef?.hide()">Cancel</Button>
							<Button type="colored" color="orange" @click="modalRef?.hide()">Proceed</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}

export const Scrollable: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Scrollable Modal</Button>
				<NewModal ref="modalRef" header="Scrollable Content" scrollable max-content-height="300px">
					<div class="space-y-4">
						<p v-for="i in 20" :key="i">
							This is paragraph {{ i }} of scrollable content. The modal will show fade indicators when scrolled.
						</p>
					</div>
					<template #actions>
						<div class="flex gap-2 justify-end">
							<Button type="colored" color="brand" @click="modalRef?.hide()">Close</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}

export const MergedHeader: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Modal (Merged Header)</Button>
				<NewModal ref="modalRef" hide-header merge-header>
					<div class="text-center py-8">
						<h2 class="text-xl font-bold mb-4">Custom Header Area</h2>
						<p>This modal has the header merged with content and only shows a close button.</p>
					</div>
				</NewModal>
			</div>
		`,
	}),
}

export const NotClosable: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Non-Closable Modal</Button>
				<NewModal ref="modalRef" header="Processing..." :closable="false" :close-on-esc="false" :close-on-click-outside="false">
					<p>This modal cannot be closed by clicking outside or pressing escape.</p>
					<p class="text-secondary mt-2">Only the action button can close it.</p>
					<template #actions>
						<div class="flex justify-end">
							<Button type="colored" color="brand" @click="modalRef?.hide()">I understand, close</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}

export const NoPadding: Story = {
	render: () => ({
		components: { NewModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
			const openModal = () => modalRef.value?.show()
			return { modalRef, openModal }
		},
		template: `
			<div>
				<Button type="colored" color="brand" @click="openModal">Open Modal (No Padding)</Button>
				<NewModal ref="modalRef" header="No Padding Modal" no-padding>
					<p>This modal has no default padding on the content area.</p>
					<template #actions>
						<div class="flex gap-2 justify-end p-6 pt-0">
							<Button type="colored" color="brand" @click="modalRef?.hide()">Close</Button>
						</div>
					</template>
				</NewModal>
			</div>
		`,
	}),
}
