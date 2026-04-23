import { RotateCounterClockwiseIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Admonition from '../../components/base/Admonition.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'

type AdmonitionType = 'info' | 'warning' | 'critical' | 'success'
type ActionType = 'Cancel' | 'Retry' | 'RetryOutlined' | 'Dismiss'
type ProgressColor = 'blue' | 'green' | 'red'

interface CopyExample {
	title: string
	body: string
	type: AdmonitionType
	action?: ActionType
	dismissible?: boolean
	progress?: number
	progressColor?: ProgressColor
	waiting?: boolean
}

interface CopySection {
	title: string
	items: CopyExample[]
}

const meta = {
	title: 'Servers/ServerPanelAdmonitionCopyDraft',
	component: Admonition,
	parameters: {
		layout: 'padded',
	},
} satisfies Meta<typeof Admonition>

export default meta
type Story = StoryObj<typeof meta>

const sections: CopySection[] = [
	{
		title: 'Installation and content sync',
		items: [
			{
				type: 'info',
				title: "We're preparing your server",
				body: 'Installing platform...',
				progress: 45,
				progressColor: 'blue',
			},
			{
				type: 'info',
				title: "We're preparing your server",
				body: 'Installing modpack...',
				progress: 72,
				progressColor: 'blue',
			},
			{
				type: 'critical',
				title: 'Installation failed',
				body: 'The specified loader or Minecraft version could not be installed. It may be invalid or unsupported.',
				action: 'RetryOutlined',
				dismissible: true,
			},
			{
				type: 'critical',
				title: 'Installation failed',
				body: 'This modpack version does not include a downloadable file. It may have been packaged incorrectly.',
				action: 'RetryOutlined',
				dismissible: true,
			},
		],
	},
	{
		title: 'Uploads and file operations',
		items: [
			{
				type: 'info',
				title: 'Uploading files (1/3) - resourcepack.zip',
				body: '20 KB / 100 KB (20%)',
				action: 'Cancel',
				progress: 0.2,
				progressColor: 'blue',
			},
			{
				type: 'info',
				title: 'Extracting story-modpack.mrpack',
				body: '2 MB extracted - overrides/server.properties',
				action: 'Cancel',
				progress: 0.35,
				progressColor: 'blue',
			},
			{
				type: 'success',
				title: 'Extracting story-modpack.mrpack - Done',
				body: '12 MB extracted',
				progress: 1,
				progressColor: 'green',
			},
			{
				type: 'critical',
				title: 'Extracting story-modpack.mrpack - Failed',
				body: '2 MB extracted',
				action: 'Dismiss',
				dismissible: true,
				progress: 0.35,
				progressColor: 'red',
			},
		],
	},
	{
		title: 'Backup creation',
		items: [
			{
				type: 'info',
				title: 'Backup queued',
				body: 'World backup is queued and will start shortly.',
				action: 'Cancel',
			},
			{
				type: 'info',
				title: 'Creating backup',
				body: 'Saving world data and server configuration for World backup. This can take a few minutes.',
				action: 'Cancel',
				progress: 0.42,
				progressColor: 'blue',
			},
			{
				type: 'critical',
				title: 'Backup failed',
				body: 'Something went wrong while creating World backup. Please try again or contact support if the issue continues.',
				action: 'Retry',
				dismissible: true,
			},
			{
				type: 'success',
				title: 'Backup completed',
				body: 'World backup finished successfully.',
				action: 'Dismiss',
				dismissible: true,
			},
		],
	},
	{
		title: 'Backup restore',
		items: [
			{
				type: 'info',
				title: 'Restore queued',
				body: 'Restoring from World backup is queued and will start shortly.',
				action: 'Cancel',
			},
			{
				type: 'info',
				title: 'Restoring from backup',
				body: 'Restoring your server from World backup. This may take a couple of minutes.',
				action: 'Cancel',
				progress: 0.65,
				progressColor: 'blue',
			},
			{
				type: 'critical',
				title: 'Restore failed',
				body: 'Something went wrong while restoring from World backup. Please try again or contact support if the issue continues.',
				action: 'Retry',
				dismissible: true,
			},
			{
				type: 'success',
				title: 'Restore complete',
				body: 'Your server has been restored to World backup and is ready to start.',
				action: 'Dismiss',
				dismissible: true,
			},
		],
	},
	{
		title: 'Busy states',
		items: [
			{
				type: 'warning',
				title: 'Background task running',
				body: 'Please wait for the operation to complete before editing content.',
			},
			{
				type: 'warning',
				title: 'Background task running',
				body: 'File operations are disabled while the operation is in progress.',
			},
		],
	},
]

export const AllCopy: Story = {
	render: () => ({
		components: { Admonition, ButtonStyled, RotateCounterClockwiseIcon },
		setup() {
			return { sections }
		},
		template: /* html */ `
			<div style="height: 100vh; overflow-y: auto; padding: 1rem 1rem 4rem 0;">
				<div style="display: flex; max-width: 840px; flex-direction: column; gap: 2rem;">
					<section v-for="section in sections" :key="section.title">
					<h2 style="margin: 0 0 0.75rem; font-size: 1.125rem; font-weight: 700;">
						{{ section.title }}
					</h2>
					<div style="display: flex; flex-direction: column; gap: 0.75rem;">
						<Admonition
							v-for="item in section.items"
							:key="item.title + item.body"
							:type="item.type"
							:header="item.title"
							:dismissible="item.dismissible"
							:progress="item.progress != null ? (item.progress > 1 ? item.progress / 100 : item.progress) : undefined"
							:progress-color="item.progressColor"
							:waiting="item.waiting"
						>
							{{ item.body }}
							<template
								v-if="
									item.action === 'Cancel' ||
										item.action === 'Retry' ||
										item.action === 'RetryOutlined'
									"
									#top-right-actions
								>
									<ButtonStyled v-if="item.action === 'Cancel'" type="outlined" color="blue">
										<button class="!border" type="button">Cancel</button>
									</ButtonStyled>
									<ButtonStyled
										v-else
										:type="item.action === 'RetryOutlined' ? 'outlined' : undefined"
										color="red"
									>
										<button
											:class="item.action === 'RetryOutlined' ? '!border' : undefined"
											type="button"
										>
											<RotateCounterClockwiseIcon class="size-5" />
											Retry
										</button>
									</ButtonStyled>
								</template>
							</Admonition>
						</div>
					</section>
				</div>
			</div>
		`,
	}),
}
