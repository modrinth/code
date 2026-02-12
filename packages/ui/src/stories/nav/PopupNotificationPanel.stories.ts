import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import PopupNotificationPanel from '../../components/nav/PopupNotificationPanel.vue'
import { injectPopupNotificationManager } from '../../providers'

const meta = {
	title: 'Nav/PopupNotificationPanel',
	component: PopupNotificationPanel,
} satisfies Meta<typeof PopupNotificationPanel>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { PopupNotificationPanel, ButtonStyled },
		setup() {
			const popupManager = injectPopupNotificationManager()

			const showSuccess = () => {
				popupManager.addPopupNotification({
					title: 'Install complete',
					text: 'Complex Gaming [Cobblemon] is installed and ready to play.',
					type: 'success',
					buttons: [
						{
							label: 'Launch game',
							action: () => console.log('Launch game clicked'),
							color: 'brand',
						},
						{
							label: 'Instance',
							action: () => console.log('Instance clicked'),
						},
					],
				})
			}

			const showError = () => {
				popupManager.addPopupNotification({
					title: 'Download failed',
					text: 'Failed to download the modpack. Please try again.',
					type: 'error',
					buttons: [
						{
							label: 'Retry',
							action: () => console.log('Retry clicked'),
							color: 'red',
						},
					],
				})
			}

			const showWarning = () => {
				popupManager.addPopupNotification({
					title: 'Update available',
					text: 'Modrinth App v2.1.0 is available now! Since you\'re on a metered network, we didn\'t automatically download it.',
					type: 'warning',
					autoCloseMs: null,
					buttons: [
						{
							label: 'Download (45 MB)',
							action: () => console.log('Download clicked'),
							color: 'brand',
						},
						{
							label: 'Changelog',
							action: () => console.log('Changelog clicked'),
						},
					],
				})
			}

			const showInfo = () => {
				popupManager.addPopupNotification({
					title: 'Download complete',
					text: 'Modrinth App v2.1.0 has finished downloading. Reload to update now.',
					type: 'info',
					buttons: [
						{
							label: 'Reload',
							action: () => console.log('Reload clicked'),
							color: 'brand',
						},
						{
							label: 'Changelog',
							action: () => console.log('Changelog clicked'),
						},
					],
				})
			}

			const showNoButtons = () => {
				popupManager.addPopupNotification({
					title: 'Heads up',
					text: 'This notification has no action buttons and will auto-close in 30 seconds.',
					type: 'info',
				})
			}

			const showPermanent = () => {
				popupManager.addPopupNotification({
					title: 'Permanent notification',
					text: 'This notification will stay open until manually dismissed.',
					type: 'warning',
					autoCloseMs: null,
				})
			}

			const clearAll = () => {
				popupManager.clearAllNotifications()
			}

			return { showSuccess, showError, showWarning, showInfo, showNoButtons, showPermanent, clearAll }
		},
		template: /* html */ `
			<div>
				<div class="flex flex-wrap gap-2">
					<ButtonStyled color="green">
						<button @click="showSuccess">Install Complete</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="showError">Download Failed</button>
					</ButtonStyled>
					<ButtonStyled color="orange">
						<button @click="showWarning">Update Available (Permanent)</button>
					</ButtonStyled>
					<ButtonStyled color="blue">
						<button @click="showInfo">Download Complete</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="showNoButtons">No Buttons</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="showPermanent">Permanent</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="clearAll">Clear All</button>
					</ButtonStyled>
				</div>
				<PopupNotificationPanel />
			</div>
		`,
	}),
}
