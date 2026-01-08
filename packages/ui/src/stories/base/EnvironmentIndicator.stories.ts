import type { Meta, StoryObj } from '@storybook/vue3-vite'

import EnvironmentIndicator from '../../components/base/EnvironmentIndicator.vue'

const meta = {
	title: 'Base/EnvironmentIndicator',
	component: EnvironmentIndicator,
} satisfies Meta<typeof EnvironmentIndicator>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		type: 'mod',
		clientSide: 'required',
		serverSide: 'optional',
	},
}

export const AllEnvironments: StoryObj = {
	render: () => ({
		components: { EnvironmentIndicator },
		template: `
			<div class="flex flex-col gap-4">
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" clientSide="required" serverSide="optional" />
					<span class="text-secondary text-sm">Client (required client, optional server)</span>
				</div>
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" clientSide="optional" serverSide="required" />
					<span class="text-secondary text-sm">Server (optional client, required server)</span>
				</div>
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" clientSide="required" serverSide="required" />
					<span class="text-secondary text-sm">Client and server (both required)</span>
				</div>
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" clientSide="optional" serverSide="optional" />
					<span class="text-secondary text-sm">Client or server (both optional)</span>
				</div>
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" clientSide="unsupported" serverSide="unsupported" />
					<span class="text-secondary text-sm">Unsupported</span>
				</div>
				<div class="flex items-center gap-2">
					<EnvironmentIndicator type="mod" typeOnly />
					<span class="text-secondary text-sm">Type only</span>
				</div>
			</div>
		`,
	}),
}
