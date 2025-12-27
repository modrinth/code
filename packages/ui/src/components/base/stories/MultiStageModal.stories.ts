import type { StoryObj } from '@storybook/vue3-vite'
import { defineComponent, h, ref } from 'vue'

import ButtonStyled from '../ButtonStyled.vue'
import type { StageConfigInput } from '../MultiStageModal.vue'
import MultiStageModal from '../MultiStageModal.vue'

const meta = {
	title: 'Base/MultiStageModal',
	component: MultiStageModal as any,
}

export default meta
type Story = StoryObj<any> & { args?: Record<string, unknown> }

// Create simple stage content components
const Stage1Content = defineComponent({
	name: 'Stage1Content',
	render() {
		return h('div', { class: 'p-4' }, [
			h('p', 'This is the first stage of the multi-stage modal.'),
			h('p', { class: 'text-secondary mt-2' }, 'Fill out the required information to proceed.'),
		])
	},
})

const Stage2Content = defineComponent({
	name: 'Stage2Content',
	render() {
		return h('div', { class: 'p-4' }, [
			h('p', 'This is the second stage.'),
			h('p', { class: 'text-secondary mt-2' }, 'Review your selections before continuing.'),
		])
	},
})

const Stage3Content = defineComponent({
	name: 'Stage3Content',
	render() {
		return h('div', { class: 'p-4' }, [
			h('p', 'This is the final stage.'),
			h('p', { class: 'text-secondary mt-2' }, 'Confirm to complete the process.'),
		])
	},
})

export const Default: Story = {
	args: {},
	render: () => ({
		components: { MultiStageModal, ButtonStyled },
		setup() {
			const modalRef = ref<any>(null)
			const context = {}

			const stages: StageConfigInput<typeof context>[] = [
				{
					id: 'step1',
					stageContent: Stage1Content,
					title: 'Step 1: Getting Started',
					leftButtonConfig: null,
					rightButtonConfig: {
						label: 'Next',
						color: 'brand',
						onClick: () => modalRef.value?.nextStage(),
					},
				},
				{
					id: 'step2',
					stageContent: Stage2Content,
					title: 'Step 2: Configuration',
					leftButtonConfig: {
						label: 'Back',
						onClick: () => modalRef.value?.prevStage(),
					},
					rightButtonConfig: {
						label: 'Next',
						color: 'brand',
						onClick: () => modalRef.value?.nextStage(),
					},
				},
				{
					id: 'step3',
					stageContent: Stage3Content,
					title: 'Step 3: Confirmation',
					leftButtonConfig: {
						label: 'Back',
						onClick: () => modalRef.value?.prevStage(),
					},
					rightButtonConfig: {
						label: 'Complete',
						color: 'brand',
						onClick: () => modalRef.value?.hide(),
					},
				},
			]

			const openModal = () => {
				modalRef.value?.show()
			}

			return { modalRef, stages, context, openModal }
		},
		template: `
			<div>
				<ButtonStyled color="brand">
					<button @click="openModal">Open Multi-Stage Modal</button>
				</ButtonStyled>
				<MultiStageModal ref="modalRef" :stages="stages" :context="context" />
			</div>
		`,
	}),
}
