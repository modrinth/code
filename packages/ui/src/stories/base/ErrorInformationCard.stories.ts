import { IssuesIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ErrorInformationCard from '../../components/base/ErrorInformationCard.vue'

const meta = {
	title: 'Base/ErrorInformationCard',
	component: ErrorInformationCard,
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="flex min-h-[400px] items-center justify-center bg-bg"><story /></div>',
		}),
	],
} satisfies Meta<typeof ErrorInformationCard>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		title: 'Something went wrong',
		description: 'An unexpected error occurred while processing your request.',
		icon: IssuesIcon,
	},
}

export const WithErrorDetails: Story = {
	args: {
		title: 'Connection Failed',
		description: 'Unable to connect to the server.',
		icon: IssuesIcon,
		errorDetails: [
			{ label: 'Error Code', value: 'ERR_CONNECTION_REFUSED', type: 'inline' },
			{ label: 'Timestamp', value: '2024-01-15T10:30:00Z', type: 'inline' },
			{
				label: 'Stack Trace',
				value: 'Error: Connection refused\n  at Socket.connect\n  at Client.connect',
				type: 'block',
			},
		],
	},
}

export const WithAction: Story = {
	args: {
		title: 'Download Failed',
		description: 'The file could not be downloaded. Please try again.',
		icon: IssuesIcon,
		action: {
			label: 'Retry Download',
			onClick: () => console.log('Retry clicked'),
			color: 'brand',
		},
	},
}
