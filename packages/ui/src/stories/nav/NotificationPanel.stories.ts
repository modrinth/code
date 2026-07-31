import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { Button } from '../../components/base/buttons'
import NotificationPanel from '../../components/nav/NotificationPanel.vue'
import { injectNotificationManager } from '../../providers'

const meta = {
	title: 'Nav/NotificationPanel',
	component: NotificationPanel,
} satisfies Meta<typeof NotificationPanel>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { NotificationPanel, Button },
		setup() {
			const notificationManager = injectNotificationManager()

			const showSuccess = () => {
				notificationManager.addNotification({
					title: 'Success',
					text: 'Your project has been published successfully.',
					type: 'success',
				})
			}

			const showError = () => {
				notificationManager.addNotification({
					title: 'An error occurred',
					text: 'Failed to upload file. Please try again.',
					type: 'error',
					errorCode: 'ERR_UPLOAD_FAILED',
				})
			}

			const showWarning = () => {
				notificationManager.addNotification({
					title: 'Warning',
					text: 'Your session is about to expire.',
					type: 'warning',
				})
			}

			const showInfo = () => {
				notificationManager.addNotification({
					title: 'Info',
					text: 'A new version of the modpack is available.',
					type: 'info',
				})
			}

			const showActions = () => {
				notificationManager.addNotification({
					title: 'Your privacy and how ads support Modrinth',
					text: 'Choose how our advertising partners may use your data.',
					type: 'neutral',
					autoCloseMs: null,
					dismissible: false,
					copyable: false,
					noIcon: true,
					buttons: [
						{
							label: 'Manage preferences',
							action: () => console.log('Manage preferences clicked'),
						},
						{
							label: 'Reject all',
							action: () => console.log('Reject all clicked'),
							color: 'brand',
						},
						{
							label: 'Accept all',
							action: () => console.log('Accept all clicked'),
							color: 'brand',
						},
					],
				})
			}

			const clearAll = () => {
				notificationManager.clearAllNotifications()
			}

			return { showSuccess, showError, showWarning, showInfo, showActions, clearAll }
		},
		template: /* html */ `
			<div>
				<div class="flex flex-wrap gap-2">
					<Button type="colored" color="green" @click="showSuccess">Success</Button>
					<Button type="colored" color="red" @click="showError">Error</Button>
					<Button type="colored" color="orange" @click="showWarning">Warning</Button>
					<Button type="colored" color="blue" @click="showInfo">Info</Button>
					<Button @click="showActions">Actions</Button>
					<Button @click="clearAll">Clear All</Button>
				</div>
				<NotificationPanel />
			</div>
		`,
	}),
}

export const LeftSide: StoryObj = {
	render: () => ({
		components: { NotificationPanel, Button },
		setup() {
			const notificationManager = injectNotificationManager()
			notificationManager.setNotificationLocation('left')

			const showSuccess = () => {
				notificationManager.addNotification({
					title: 'Success',
					text: 'Your project has been published successfully.',
					type: 'success',
				})
			}

			const showError = () => {
				notificationManager.addNotification({
					title: 'An error occurred',
					text: 'Failed to upload file. Please try again.',
					type: 'error',
					errorCode: 'ERR_UPLOAD_FAILED',
				})
			}

			const showWarning = () => {
				notificationManager.addNotification({
					title: 'Warning',
					text: 'Your session is about to expire.',
					type: 'warning',
				})
			}

			const showInfo = () => {
				notificationManager.addNotification({
					title: 'Info',
					text: 'A new version of the modpack is available.',
					type: 'info',
				})
			}

			const clearAll = () => {
				notificationManager.clearAllNotifications()
			}

			return { showSuccess, showError, showWarning, showInfo, clearAll }
		},
		template: /* html */ `
			<div>
				<div class="flex flex-wrap gap-2">
					<Button type="colored" color="green" @click="showSuccess">Success</Button>
					<Button type="colored" color="red" @click="showError">Error</Button>
					<Button type="colored" color="orange" @click="showWarning">Warning</Button>
					<Button type="colored" color="blue" @click="showInfo">Info</Button>
					<Button @click="clearAll">Clear All</Button>
				</div>
				<NotificationPanel />
			</div>
		`,
	}),
}
