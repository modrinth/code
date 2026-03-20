import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Admonition from '../../components/base/Admonition.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ProgressBar from '../../components/base/ProgressBar.vue'

const meta = {
	title: 'Base/Admonition',
	component: Admonition,
} satisfies Meta<typeof Admonition>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		body: 'This is an informational message.',
	},
}

export const AllTypes: Story = {
	render: () => ({
		components: { Admonition },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<Admonition type="info" header="Info" body="This is an informational message." />
				<Admonition type="warning" header="Warning" body="This is a warning message." />
				<Admonition type="critical" header="Critical" body="This is a critical message." />
				<Admonition type="success" header="Success" body="This operation completed successfully." />
			</div>
		`,
	}),
}

export const WithHeader: Story = {
	args: {
		type: 'warning',
		header: 'Important Notice',
		body: 'Please read this carefully before proceeding.',
	},
}

export const Success: Story = {
	args: {
		type: 'success',
		header: 'Operation Complete',
		body: 'Everything went smoothly.',
	},
}

export const Dismissible: Story = {
	args: {
		type: 'info',
		header: 'Dismissible Notice',
		body: 'This admonition can be dismissed by clicking the X button.',
		dismissible: true,
	},
}

export const WithTopRightActions: Story = {
	render: () => ({
		components: { Admonition, ButtonStyled },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<Admonition type="info" header="Uploading files (2/5)">
					Uploading server files...
					<template #top-right-actions>
						<ButtonStyled type="outlined" color="blue">
							<button class="!border">Cancel</button>
						</ButtonStyled>
					</template>
				</Admonition>
				<Admonition type="critical" header="Extraction failed">
					Something went wrong while extracting the archive.
					<template #top-right-actions>
						<ButtonStyled color="red">
							<button>Retry</button>
						</ButtonStyled>
						<ButtonStyled circular type="transparent" hover-color-fill="background" color="red">
							<button>✕</button>
						</ButtonStyled>
					</template>
				</Admonition>
				<Admonition type="success" header="Extraction complete">
					All files have been extracted successfully.
					<template #top-right-actions>
						<ButtonStyled circular type="transparent" hover-color-fill="background" color="green">
							<button>✕</button>
						</ButtonStyled>
					</template>
				</Admonition>
			</div>
		`,
	}),
}

export const WithProgressBar: Story = {
	render: () => ({
		components: { Admonition, ButtonStyled, ProgressBar },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<Admonition type="info" header="Uploading files (2/5)">
					128 KB / 1.2 MB (45%)
					<template #top-right-actions>
						<ButtonStyled type="outlined" color="blue">
							<button class="!border">Cancel</button>
						</ButtonStyled>
					</template>
					<template #progress>
						<ProgressBar :progress="0.45" :max="1" color="blue" full-width />
					</template>
				</Admonition>
				<Admonition type="info" header="Extracting modpack.zip">
					24 MB extracted — config/settings.yml
					<template #top-right-actions>
						<ButtonStyled type="outlined" color="blue">
							<button class="!border">Cancel</button>
						</ButtonStyled>
					</template>
					<template #progress>
						<ProgressBar :progress="0.7" :max="1" color="blue" full-width />
					</template>
				</Admonition>
				<Admonition type="success" header="Extraction complete — Done">
					56 MB extracted
					<template #top-right-actions>
						<ButtonStyled circular type="transparent" hover-color-fill="background" color="green">
							<button>✕</button>
						</ButtonStyled>
					</template>
					<template #progress>
						<ProgressBar :progress="1" :max="1" color="green" full-width />
					</template>
				</Admonition>
			</div>
		`,
	}),
}
