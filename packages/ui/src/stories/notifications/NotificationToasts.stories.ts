import { CopyIcon, MinecraftServerIcon, UpdatedIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { NotificationToast } from '../../components/notifications'
import type { PopupNotificationButton } from '../../providers'

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
					:entity-icon-url="instanceIconUrl"
					entity-name="New Creation"
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
					progress-type="percentage"
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

export const DownloadProgressLabels: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				instanceIconUrl: MinecraftServerIcon,
				noop,
			}
		},
		template: /* html */ `
			<div class="flex flex-col gap-4">
				<NotificationToast
					type="instance-download"
					entity-name="Cobblemon Official Modpack"
					:entity-icon-url="instanceIconUrl"
					status-text="Downloading pack file"
					progress-type="bytes"
					:progress-current="128 * 1024 * 1024"
					:progress-total="512 * 1024 * 1024"
					:progress="0.25"
					@dismiss="noop"
				/>
				<NotificationToast
					type="instance-download"
					entity-name="Cobblemon Official Modpack"
					:entity-icon-url="instanceIconUrl"
					status-text="Downloading content"
					progress-type="count"
					:progress-current="3"
					:progress-total="193"
					:progress="3 / 193"
					@dismiss="noop"
				/>
				<NotificationToast
					type="instance-download"
					entity-name="New Creation"
					:entity-icon-url="instanceIconUrl"
					status-text="Downloading Minecraft"
					progress-type="percentage"
					:progress="0.63"
					@dismiss="noop"
				/>
			</div>
		`,
	}),
}

export const FailedDownloadActions: Story = {
	render: () => ({
		components: { NotificationToast },
		setup() {
			return {
				instanceIconUrl: MinecraftServerIcon,
				actions: [
					{
						label: 'Retry',
						icon: UpdatedIcon,
						color: 'brand',
						action: noop,
					},
					{
						label: 'Copy details',
						icon: CopyIcon,
						color: 'standard',
						action: noop,
					},
				] satisfies PopupNotificationButton[],
				noop,
			}
		},
		template: /* html */ `
			<NotificationToast
				type="instance-download"
				entity-name="Cobblemon Official Modpack"
				:entity-icon-url="instanceIconUrl"
				status-text="Failed while downloading content."
				:progress="0"
				:show-progress="false"
				wrap-text
				:actions="actions"
				@action="(index) => actions[index].action()"
				@dismiss="noop"
			/>
		`,
	}),
}
