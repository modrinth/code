import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerNotice from '../../components/base/ServerNotice.vue'

const meta = {
	title: 'Base/ServerNotice',
	component: ServerNotice,
} satisfies Meta<typeof ServerNotice>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		level: 'info',
		message: 'This is an informational server notice.',
		dismissable: true,
	},
}

export const AllLevels: StoryObj = {
	render: () => ({
		components: { ServerNotice },
		template: `
			<div class="flex flex-col gap-4">
				<ServerNotice
					level="info"
					message="This is an informational notice for users."
					:dismissable="true"
					@dismiss="() => console.log('dismissed')"
				/>
				<ServerNotice
					level="warn"
					message="This is a warning notice that requires attention."
					:dismissable="true"
					@dismiss="() => console.log('dismissed')"
				/>
				<ServerNotice
					level="critical"
					message="This is a critical notice about an important issue."
					:dismissable="true"
					@dismiss="() => console.log('dismissed')"
				/>
			</div>
		`,
	}),
}

export const WithTitle: Story = {
	args: {
		level: 'warn',
		message: 'Server maintenance is scheduled for tonight at midnight.',
		dismissable: true,
		title: 'Scheduled Maintenance',
	},
}

export const NonDismissable: Story = {
	args: {
		level: 'critical',
		message: 'Your account requires verification before you can upload projects.',
		dismissable: false,
	},
}
