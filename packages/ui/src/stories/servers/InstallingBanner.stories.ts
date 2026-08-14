import type { Meta, StoryObj } from '@storybook/vue3-vite'

import InstallingBanner from '../../components/servers/InstallingBanner.vue'

const meta = {
	title: 'Servers/InstallingBanner',
	component: InstallingBanner,
} satisfies Meta<typeof InstallingBanner>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	name: 'Default (no progress)',
}

export const WithProgress: Story = {
	args: {
		progress: {
			phase: 'InstallingLoader',
			percent: 45,
		},
	},
}

export const IndeterminateLoaderInstall: Story = {
	args: {
		progress: {
			phase: 'InstallingLoader',
			percent: 0,
		},
	},
}

export const InstallingModpack: Story = {
	args: {
		progress: {
			phase: 'InstallingPack',
			percent: 72,
		},
	},
}

export const InstallingAddons: Story = {
	args: {
		progress: {
			phase: 'Addons',
			percent: 90,
		},
	},
}

export const ErrorInvalidVersion: Story = {
	name: 'Error: Invalid Version',
	args: {
		contentError: {
			step: 'modloader',
			description: 'the specified version may be incorrect',
		},
	},
}

export const ErrorUnsupportedVersion: Story = {
	name: 'Error: Unsupported Version',
	args: {
		contentError: {
			step: 'modloader',
			description: 'this version is not yet supported',
		},
	},
}

export const ErrorInternal: Story = {
	name: 'Error: Internal',
	args: {
		contentError: {
			step: 'modloader',
			description: 'internal error',
		},
	},
}

export const ErrorModpackInstall: Story = {
	name: 'Error: Modpack Install Failed',
	args: {
		contentError: {
			step: 'modpack',
			description: 'Failed to install modpack',
		},
	},
}

export const ErrorModpackNoFile: Story = {
	name: 'Error: Modpack No Primary File',
	args: {
		contentError: {
			step: 'modpack',
			description: 'Modpack version has no primary file',
		},
	},
}

export const AllStates: Story = {
	render: () => ({
		components: { InstallingBanner },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<InstallingBanner />
				<InstallingBanner :progress="{ phase: 'InstallingLoader', percent: 0 }" />
				<InstallingBanner :progress="{ phase: 'InstallingLoader', percent: 45 }" />
				<InstallingBanner :content-error="{ step: 'modloader', description: 'the specified version may be incorrect' }" />
				<InstallingBanner :content-error="{ step: 'modloader', description: 'this version is not yet supported' }" />
				<InstallingBanner :content-error="{ step: 'modloader', description: 'internal error' }" />
				<InstallingBanner :content-error="{ step: 'modpack', description: 'Failed to install modpack' }" />
			</div>
		`,
	}),
}
