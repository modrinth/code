import { MinecraftServerIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { NotificationToast } from '../../components/notifications'

const avatarUrl =
	'https://cdn.modrinth.com/user/6Qo4A5QT/9d81be1a9fb1afd163b7f2f05a791955e7693c90.png'

const meta = {
	title: 'Notifications/Toasts',
	parameters: {
		layout: 'centered',
	},
	decorators: [
		(story) => ({
			components: { story },
			template:
				'<div class="dark-mode flex w-[436px] flex-col gap-4 bg-surface-1 p-2"><story /></div>',
		}),
	],
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

const noop = () => console.log('Notification action')

export const FigmaExamples: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				avatarUrl,
				instanceIconUrl: MinecraftServerIcon,
				noop,
			}
		},
		template: /* html */ `
			<div class="flex flex-col gap-4">
				<NotificationToast
					type="friend-request"
					actor-name="Fetch"
					:actor-avatar-url="avatarUrl"
					@accept="noop"
					@decline="noop"
					@dismiss="noop"
				/>
				<NotificationToast
					type="server-invite"
					actor-name="Fetch"
					:actor-avatar-url="avatarUrl"
					entity-name="Modrinth SMP"
					@accept="noop"
					@decline="noop"
					@dismiss="noop"
					@open-actor="noop"
				/>
				<NotificationToast
					type="instance-invite"
					actor-name="Fetch"
					:actor-avatar-url="avatarUrl"
					entity-name="New Creation"
					:entity-icon-url="instanceIconUrl"
					@accept="noop"
					@decline="noop"
					@dismiss="noop"
					@open-actor="noop"
				/>
				<NotificationToast
					type="instance-download"
					entity-name="New Creation"
					:entity-icon-url="instanceIconUrl"
					status-text="Downloading Minecraft..."
					:progress="0.5"
					@dismiss="noop"
				/>
				<NotificationToast
					type="instance-ready"
					entity-name="New Creation"
					:entity-icon-url="instanceIconUrl"
					:progress="0.75"
					@launch="noop"
					@open-instance="noop"
					@dismiss="noop"
				/>
			</div>
		`,
	}),
}

export const ServerInvite: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				avatarUrl,
				instanceIconUrl: MinecraftServerIcon,
				noop,
			}
		},
		template: /* html */ `
			<NotificationToast
				type="server-invite"
				actor-name="Fetch"
				:actor-avatar-url="avatarUrl"
				entity-name="Modrinth SMP"
				@accept="noop"
				@decline="noop"
				@dismiss="noop"
				@open-actor="noop"
			/>
		`,
	}),
}

export const MissingAvatarFallback: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				instanceIconUrl: MinecraftServerIcon,
				noop,
			}
		},
		template: /* html */ `
			<NotificationToast
				type="server-invite"
				actor-name="Fetch"
				entity-name="Modrinth SMP"
				@accept="noop"
				@decline="noop"
				@dismiss="noop"
				@open-actor="noop"
			/>
		`,
	}),
}

export const WaitingDownload: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				instanceIconUrl: MinecraftServerIcon,
				noop,
			}
		},
		template: /* html */ `
			<NotificationToast
				type="instance-download"
				entity-name="New Creation"
				:entity-icon-url="instanceIconUrl"
				status-text="Preparing files..."
				waiting
				@dismiss="noop"
			/>
		`,
	}),
}
