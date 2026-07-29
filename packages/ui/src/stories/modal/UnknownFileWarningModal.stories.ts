import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import UnknownFileWarningModal from '../../components/modal/UnknownFileWarningModal.vue'

const meta = {
	title: 'Modal/UnknownFileWarningModal',
	component: UnknownFileWarningModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof UnknownFileWarningModal>

export default meta
type Story = StoryObj<typeof meta>

export const Modpack: Story = {
	render: () => ({
		components: { ButtonStyled, UnknownFileWarningModal },
		setup() {
			const modalRef = ref<InstanceType<typeof UnknownFileWarningModal> | null>(null)
			return { modalRef }
		},
		template: /* html */ `
			<ButtonStyled color="brand">
				<button @click="modalRef?.show()">Open modpack warning</button>
			</ButtonStyled>
			<UnknownFileWarningModal
				ref="modalRef"
				mode="modpack"
				file-name="cozy-cottage-1.4.0.mrpack"
				:external-files-in-modpack="[
					'voicechat-fabric-1.20.1-2.5.26.jar',
					'xaeros-minimap-24.6.1_Fabric_1.20.jar',
					'InventoryProfilesNext-forge-1.20.1-1.10.12.jar',
					'MouseTweaks-forge-mc1.20.1-2.25.jar',
					'Terralith_1.20.x_v2.5.4.jar',
					'YungsApi-1.20-Forge-4.0.5.jar',
				]"
			/>
		`,
	}),
}

export const Mod: Story = {
	render: () => ({
		components: { ButtonStyled, UnknownFileWarningModal },
		setup() {
			const modalRef = ref<InstanceType<typeof UnknownFileWarningModal> | null>(null)
			return { modalRef }
		},
		template: /* html */ `
			<ButtonStyled color="brand">
				<button @click="modalRef?.show()">Open file warning</button>
			</ButtonStyled>
			<UnknownFileWarningModal
				ref="modalRef"
				mode="mod"
				file-name="voicechat-fabric-1.20.1-2.5.26.jar"
			/>
		`,
	}),
}
