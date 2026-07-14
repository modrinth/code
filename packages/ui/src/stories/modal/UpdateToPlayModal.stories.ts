import { DownloadIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentDiffModal from '../../layouts/shared/installation-settings/components/ContentDiffModal.vue'
import type { ContentDiffItem } from '../../layouts/shared/installation-settings/types'

const meta = {
	title: 'Modal/UpdateToPlayModal',
	component: ContentDiffModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ContentDiffModal>

export default meta
type Story = StoryObj<typeof meta>

const diffs: ContentDiffItem[] = [
	{
		type: 'added',
		external: true,
		fileName: 'voicechat-fabric-1.20.1-2.5.26.jar',
	},
	{
		type: 'added',
		external: true,
		fileName: 'xaeros-minimap-24.6.1_Fabric_1.20.jar',
	},
	{
		type: 'updated',
		projectName: 'Cloth Config API',
		currentVersionName: '18.0.145+neoforge',
		newVersionName: '20.0.149+neoforge',
	},
	{
		type: 'added',
		projectName: 'Sodium',
		newVersionName: '1.21.10-0.7.3-neoforge',
	},
	{
		type: 'updated',
		projectName: 'Iris Shaders',
		currentVersionName: '1.8.8+1.21.8-neoforge',
		newVersionName: '1.9.6+1.21.10-neoforge',
	},
	{
		type: 'updated',
		projectName: 'Entity Culling',
		currentVersionName: '1.8.1',
		newVersionName: '1.9.3',
	},
	{
		type: 'updated',
		projectName: 'FerriteCore',
		currentVersionName: '7.0.2',
		newVersionName: '8.0.0',
	},
	{
		type: 'removed',
		projectName: 'Lithium',
		currentVersionName: '0.15.0+mc1.21.8',
	},
]

export const ExternalFiles: Story = {
	render: () => ({
		components: { ButtonStyled, ContentDiffModal },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDiffModal> | null>(null)
			return { diffs, DownloadIcon, modalRef }
		},
		template: /* html */ `
			<ButtonStyled color="brand">
				<button @click="modalRef?.show()">Open update warning</button>
			</ButtonStyled>
			<ContentDiffModal
				ref="modalRef"
				header="Update to play"
				description="An update is required to play Epic Modrinth Pack. Please update to the latest version to launch the game."
				:diffs="diffs"
				version-date="November 25, 2025"
				show-external-warnings
				confirm-label="Update"
				:confirm-icon="DownloadIcon"
				removed-label="Removed"
			/>
		`,
	}),
}
