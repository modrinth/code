import { DownloadIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
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

const description =
	'An update is required to play Epic Modrinth Pack. Please update to the latest version to launch the game.'

const regularDiffs: ContentDiffItem[] = [
	{
		type: 'updated',
		projectName: 'Fabric API',
		currentVersionName: '0.128.2+1.21.10',
		newVersionName: '0.139.2+1.21.11',
	},
	{
		type: 'removed',
		projectName: 'Optifine',
		currentVersionName: '1.0.2-forge-mc1.14.4',
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
]

export const NoExternalFiles: Story = {
	render: () => ({
		components: { ContentDiffModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDiffModal> | null>(null)
			return { diffs: regularDiffs, description, DownloadIcon, modalRef }
		},
		template: /* html */ `
			<Button type="colored" color="brand" @click="modalRef?.show()">Open update</Button>
			<ContentDiffModal
				ref="modalRef"
				header="Update to play"
				:description="description"
				:diffs="diffs"
				version-date="November 25, 2025"
				confirm-label="Install"
				:confirm-icon="DownloadIcon"
				removed-label="Removed"
			/>
		`,
	}),
}

function externalFilesStory(name: string, offerSandbox: boolean): Story {
	return {
		name,
		render: () => ({
			components: { ContentDiffModal, Button },
			setup() {
				const modalRef = ref<InstanceType<typeof ContentDiffModal> | null>(null)
				return { diffs, description, offerSandbox, DownloadIcon, modalRef }
			},
			template: /* html */ `
				<Button type="colored" color="brand" @click="modalRef?.show()">Open update warning</Button>
				<ContentDiffModal
					ref="modalRef"
					header="Update to play"
					:description="description"
					:diffs="diffs"
					version-date="November 25, 2025"
					show-external-warnings
					:offer-sandbox="offerSandbox"
					confirm-label="Update"
					:confirm-icon="DownloadIcon"
					removed-label="Removed"
				/>
			`,
		}),
	}
}

export const ExternalFilesSandboxOff = externalFilesStory('External files (sandbox off)', true)
export const ExternalFilesSandboxOn = externalFilesStory(
	'External files (sandbox on or unavailable)',
	false,
)

const configFileNames = [
	'options.txt',
	'config/sodium-options.json',
	'config/iris.properties',
	'config/create/client.toml',
	'config/create/common.toml',
	'config/create/server.toml',
	'config/jei/jei-client.ini',
	'config/jei/blacklist.cfg',
	'defaultconfigs/ftbquests/quests/data.snbt',
	'kubejs/server_scripts/recipes.js',
]

const configDiffs: ContentDiffItem[] = [
	{
		type: 'updated',
		projectName: 'FerriteCore',
		currentVersionName: '7.0.2',
		newVersionName: '8.0.0',
	},
	{
		type: 'config_files_updated',
		fileCount: configFileNames.length,
		fileNames: configFileNames,
	},
]

export const ConfigFilesOnly: Story = {
	render: () => ({
		components: { ContentDiffModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDiffModal> | null>(null)
			const diffs = configDiffs.slice(1)
			return { diffs, DownloadIcon, modalRef }
		},
		template: /* html */ `
			<Button type="colored" color="brand" @click="modalRef?.show()">Open update warning</Button>
			<ContentDiffModal
				ref="modalRef"
				header="Update to play"
				admonition-header="Update required"
				description="An update is required to play server play 3 (public). Please update to latest version to launch the game."
				:diffs="diffs"
				confirm-label="Update"
				:confirm-icon="DownloadIcon"
			/>
		`,
	}),
}

export const ConfigFilesWithContent: Story = {
	render: () => ({
		components: { ContentDiffModal, Button },
		setup() {
			const modalRef = ref<InstanceType<typeof ContentDiffModal> | null>(null)
			return { diffs: configDiffs, DownloadIcon, modalRef }
		},
		template: /* html */ `
			<Button type="colored" color="brand" @click="modalRef?.show()">Open update warning</Button>
			<ContentDiffModal
				ref="modalRef"
				header="Update to play"
				admonition-header="Update required"
				description="An update is required to play server play 3 (public). Please update to latest version to launch the game."
				:diffs="diffs"
				confirm-label="Update"
				:confirm-icon="DownloadIcon"
			/>
		`,
	}),
}
