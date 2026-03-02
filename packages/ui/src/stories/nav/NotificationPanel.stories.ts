import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import NotificationPanel from '../../components/nav/NotificationPanel.vue'
import { injectNotificationManager } from '../../providers'

const meta = {
	title: 'Nav/NotificationPanel',
	component: NotificationPanel,
} satisfies Meta<typeof NotificationPanel>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { NotificationPanel, ButtonStyled },
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

			const clearAll = () => {
				notificationManager.clearAllNotifications()
			}

			return { showSuccess, showError, showWarning, showInfo, clearAll }
		},
		template: /* html */ `
			<div>
				<div class="flex flex-wrap gap-2">
					<ButtonStyled color="green">
						<button @click="showSuccess">Success</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="showError">Error</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
						<button @click="showWarning">Warning</button>
					</ButtonStyled>
					<ButtonStyled color="blue">
						<button @click="showInfo">Info</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="clearAll">Clear All</button>
					</ButtonStyled>
				</div>
				<NotificationPanel />
			</div>
		`,
	}),
}

export const LeftSide: StoryObj = {
	render: () => ({
		components: { NotificationPanel, ButtonStyled },
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
					<ButtonStyled color="green">
						<button @click="showSuccess">Success</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="showError">Error</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
						<button @click="showWarning">Warning</button>
					</ButtonStyled>
					<ButtonStyled color="blue">
						<button @click="showInfo">Info</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="clearAll">Clear All</button>
					</ButtonStyled>
				</div>
				<NotificationPanel />
			</div>
		`,
	}),
}
