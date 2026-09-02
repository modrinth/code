import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { Button } from '../../components/base/buttons'
import PopupNotificationPanel from '../../components/nav/PopupNotificationPanel.vue'
import { injectPopupNotificationManager } from '../../providers'

const meta = {
	title: 'Nav/PopupNotificationPanel',
	component: PopupNotificationPanel,
} satisfies Meta<typeof PopupNotificationPanel>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { PopupNotificationPanel, Button },
		setup() {
			const popupManager = injectPopupNotificationManager()

			const showSuccess = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
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
					contentType: 'standard',
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
					contentType: 'standard',
					title: 'Update available',
					text: "Modrinth App v2.1.0 is available now! Since you're on a metered network, we didn't automatically download it.",
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
					contentType: 'standard',
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
					contentType: 'standard',
					title: 'Heads up',
					text: 'This notification has no action buttons and will auto-close in 30 seconds.',
					type: 'info',
				})
			}

			const showPermanent = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
					title: 'Permanent notification',
					text: 'This notification will stay open until manually dismissed.',
					type: 'warning',
					autoCloseMs: null,
				})
			}

			const showBlocking = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
					title: 'Your privacy and how ads support Modrinth',
					text: 'Ads make Modrinth possible and fund creator rewards. Our partners may store unique identifiers to personalize ads and measure performance.',
					type: 'info',
					autoCloseMs: null,
					dismissible: false,
					buttons: [
						{
							label: 'Manage preferences',
							action: () => console.log('Manage preferences clicked'),
							color: 'standard',
							keepOpen: true,
						},
						{
							label: 'Reject all',
							action: () => console.log('Reject all clicked'),
							color: 'brand',
							keepOpen: true,
						},
						{
							label: 'Accept all',
							action: () => console.log('Accept all clicked'),
							color: 'brand',
							keepOpen: true,
						},
					],
				})
			}

			const showWaitingProgress = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
					title: 'Installing modpack...',
					text: 'example-pack-1.0.0.mrpack',
					type: 'info',
					autoCloseMs: null,
					waiting: true,
				})
			}

			const showDeterminateProgress = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
					title: 'Downloading update',
					text: 'Downloading files...',
					type: 'success',
					autoCloseMs: null,
					progress: 0.62,
				})
			}

			const showGroupedDownloads = () => {
				popupManager.addPopupNotification({
					contentType: 'standard',
					title: 'Downloads',
					type: 'download',
					autoCloseMs: null,
					progressItems: [
						{
							id: 'java-21',
							title: 'Downloading Java 21',
							text: '42% Downloading runtime',
							progress: 0.42,
							waiting: false,
						},
						{
							id: 'pack-nebula',
							title: 'Nebula Pack',
							text: '8% Resolving files',
							progress: 0.08,
							waiting: false,
						},
						{
							id: 'assets',
							title: 'Assets',
							text: 'Preparing...',
							progress: 0,
							waiting: true,
						},
					],
				})
			}

			const clearAll = () => {
				popupManager.clearAllNotifications()
			}

			return {
				showSuccess,
				showError,
				showWarning,
				showInfo,
				showNoButtons,
				showPermanent,
				showBlocking,
				showWaitingProgress,
				showDeterminateProgress,
				showGroupedDownloads,
				clearAll,
			}
		},
		template: /* html */ `
			<div>
				<div class="flex flex-wrap gap-2">
					<Button type="colored" color="green" @click="showSuccess">Install Complete</Button>
					<Button type="colored" color="red" @click="showError">Download Failed</Button>
					<Button type="colored" color="orange" @click="showWarning">Update Available (Permanent)</Button>
					<Button type="colored" color="blue" @click="showInfo">Download Complete</Button>
					<Button @click="showNoButtons">No Buttons</Button>
					<Button @click="showPermanent">Permanent</Button>
					<Button @click="showBlocking">Blocking</Button>
					<Button @click="showWaitingProgress">Waiting Progress</Button>
					<Button @click="showDeterminateProgress">Determinate Progress</Button>
					<Button @click="showGroupedDownloads">Grouped Downloads</Button>
					<Button @click="clearAll">Clear All</Button>
				</div>
				<PopupNotificationPanel />
			</div>
		`,
	}),
}
