import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'

import ManagedContentCard from '../../layouts/shared/content-tab/components/managed-content-card/index.vue'
import type { ManagedContentCardData } from '../../layouts/shared/content-tab/types'

const cobblemonIcon =
	'https://cdn.modrinth.com/data/5FFgwNNP/e7f9ee2e9d361623847853fe2ddce42f519ee64f.png'
const serverIcon =
	'https://cdn.modrinth.com/data/PEMTFQFO/f4140c3a9042e4a825d47dbb64889dad9fdc066c.png'
const userAvatar = 'https://cdn.modrinth.com/user/LnK8MbX7/icon.png'

const figmaSummary = [
	{ type: 'mod', count: 52 },
	{ type: 'resourcepack', count: 2 },
	{ type: 'shader', count: 1 },
] satisfies ManagedContentCardData['summary']

const twoDaysAgo = new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString()
const fiveMinutesAgo = Date.now() - 5 * 60 * 1000

const modpackCard = {
	kind: 'modpack',
	manager: {
		name: 'Cobblemon Modpack',
		iconUrl: cobblemonIcon,
		link: '/modpack/cobblemon-fabric',
	},
	summary: figmaSummary,
	versionNumber: 'v1.6.1',
	versionLink: '/modpack/cobblemon-fabric/version/v1.6.1',
	updatedAt: twoDaysAgo,
} satisfies ManagedContentCardData

const serverCard = {
	kind: 'server',
	manager: {
		name: 'Modrinth SMP',
		iconUrl: serverIcon,
		link: '/server/modrinth-smp',
	},
	summary: figmaSummary,
	syncedAt: fiveMinutesAgo,
	updateAvailable: true,
} satisfies ManagedContentCardData

const sharedInstanceCard = {
	kind: 'shared-instance',
	manager: {
		name: 'Fetch',
		iconUrl: userAvatar,
		link: '/user/fetch',
	},
	summary: figmaSummary,
	syncedAt: fiveMinutesAgo,
	updateAvailable: true,
} satisfies ManagedContentCardData

const meta = {
	title: 'Instances/ManagedContentCard',
	component: ManagedContentCard,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="w-[1020px] max-w-full"><story /></div>',
		}),
	],
	args: {
		showViewContent: true,
		showSettings: true,
		showPrimaryAction: true,
		onViewContent: fn(),
		onSettings: fn(),
		onPrimaryAction: fn(),
	},
} satisfies Meta<typeof ManagedContentCard>

export default meta
type Story = StoryObj<typeof meta>

export const Modpack: Story = {
	args: {
		data: modpackCard,
	},
}

export const Server: Story = {
	args: {
		data: serverCard,
	},
}

export const SharedInstance: Story = {
	args: {
		data: sharedInstanceCard,
	},
}

export const NarrowWidth: Story = {
	args: {
		data: serverCard,
	},
	render: (args) => ({
		components: { ManagedContentCard },
		setup: () => ({ args }),
		template: /*html*/ `
			<div class="w-[620px] max-w-full">
				<ManagedContentCard v-bind="args" />
			</div>
		`,
	}),
}

export const LongManagerName: Story = {
	args: {
		data: {
			...sharedInstanceCard,
			manager: {
				...sharedInstanceCard.manager,
				name: 'A shared-instance manager with a very long display name',
			},
		},
	},
}

export const MissingManagerIcon: Story = {
	args: {
		data: {
			...serverCard,
			manager: {
				name: 'Modrinth SMP',
			},
		},
	},
}

export const ImportedModpack: Story = {
	args: {
		data: {
			kind: 'modpack',
			manager: {
				name: 'Imported from Prism Launcher',
			},
			summary: figmaSummary,
		},
		showPrimaryAction: false,
	},
}

export const LoadingCounts: Story = {
	args: {
		data: {
			...modpackCard,
			summary: undefined,
		},
	},
}

export const ZeroContent: Story = {
	args: {
		data: {
			...sharedInstanceCard,
			summary: [],
			updateAvailable: false,
		},
		showPrimaryAction: false,
	},
}

export const ApplyingUpdate: Story = {
	args: {
		data: serverCard,
		disabled: true,
		disabledText: 'Updating...',
	},
}

export const AccountLocked: Story = {
	args: {
		data: {
			...sharedInstanceCard,
			updateAvailable: false,
		},
		showPrimaryAction: false,
	},
}

export const EveryContentType: Story = {
	args: {
		data: {
			...serverCard,
			summary: [
				{ type: 'mod', count: 52 },
				{ type: 'plugin', count: 4 },
				{ type: 'datapack', count: 3 },
				{ type: 'resourcepack', count: 2 },
				{ type: 'shader', count: 1 },
			],
		},
	},
}
