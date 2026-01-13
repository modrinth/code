<script setup>
import { AuthFeature, PanelVersionFeature, TauriModrinthClient } from '@modrinth/api-client'
import {
	ArrowBigUpDashIcon,
	ChangeSkinIcon,
	CompassIcon,
	DownloadIcon,
	ExternalIcon,
	HomeIcon,
	LeftArrowIcon,
	LibraryIcon,
	LogInIcon,
	LogOutIcon,
	MaximizeIcon,
	MinimizeIcon,
	NewspaperIcon,
	NotepadTextIcon,
	PlusIcon,
	RefreshCwIcon,
	RestoreIcon,
	RightArrowIcon,
	ServerIcon,
	SettingsIcon,
	UserIcon,
	WorldIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewsArticleCard,
	NotificationPanel,
	OverflowMenu,
	ProgressSpinner,
	provideModrinthClient,
	provideNotificationManager,
	providePageContext,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { $fetch } from 'ofetch'
import { computed, onMounted, onUnmounted, provide, ref, watch } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import ModrinthAppLogo from '@/assets/modrinth_app.svg?component'
import ModrinthLoadingIndicator from '@/components/LoadingIndicatorBar.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import FriendsList from '@/components/ui/friends/FriendsList.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import AppSettingsModal from '@/components/ui/modal/AppSettingsModal.vue'
import AuthGrantFlowWaitModal from '@/components/ui/modal/AuthGrantFlowWaitModal.vue'
import NavButton from '@/components/ui/NavButton.vue'
import PromotionWrapper from '@/components/ui/PromotionWrapper.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import UpdateToast from '@/components/ui/UpdateToast.vue'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import { hide_ads_window, init_ads_window, show_ads_window } from '@/helpers/ads.js'
import { debugAnalytics, initAnalytics, optOutAnalytics, trackEvent } from '@/helpers/analytics'
import { check_reachable } from '@/helpers/auth.js'
import { get_user } from '@/helpers/cache.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { useFetch } from '@/helpers/fetch.js'
import { cancelLogin, get as getCreds, login, logout } from '@/helpers/mr_auth.ts'
import { list } from '@/helpers/profile.js'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { get_opening_command, initialize_state } from '@/helpers/state'
import {
	areUpdatesEnabled,
	enqueueUpdateForInstallation,
	getOS,
	getUpdateSize,
	isDev,
	isNetworkMetered,
} from '@/helpers/utils.js'
import i18n from '@/i18n.config'
import {
	provideAppUpdateDownloadProgress,
	subscribeToDownloadProgress,
} from '@/providers/download-progress.ts'
import { useError } from '@/store/error.js'
import { useInstall } from '@/store/install.js'
import { useLoading, useTheming } from '@/store/state'

import { create_profile_and_install_from_file } from './helpers/pack'
import { generateSkinPreviews } from './helpers/rendering/batch-skin-renderer'
import { get_available_capes, get_available_skins } from './helpers/skins'
import { AppNotificationManager } from './providers/app-notifications'

const themeStore = useTheming()

const notificationManager = new AppNotificationManager()
provideNotificationManager(notificationManager)
const { handleError, addNotification } = notificationManager

const tauriApiClient = new TauriModrinthClient({
	userAgent: `modrinth/theseus/${getVersion()} (support@modrinth.com)`,
	features: [
		new AuthFeature({
			token: async () => (await getCreds()).session,
		}),
		new PanelVersionFeature(),
	],
})
provideModrinthClient(tauriApiClient)
providePageContext({
	hierarchicalSidebarAvailable: ref(true),
	showAds: ref(false),
})
const news = ref([])
const availableSurvey = ref(false)

const urlModal = ref(null)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const os = ref('')

const stateInitialized = ref(false)

const criticalErrorMessage = ref()

const isMaximized = ref(false)

const authUnreachableDebug = useDebugLogger('AuthReachableChecker')
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		authUnreachableDebug('Auth servers are reachable')
		return true
	},
	refetchInterval: 5 * 60 * 1000, // 5 minutes
	retry: false,
	refetchOnWindowFocus: false,
})

const authUnreachable = computed(() => {
	if (authServerQuery.isError.value && !authServerQuery.isLoading.value) {
		console.warn('Failed to reach auth servers', authServerQuery.error.value)
		return true
	}
	return false
})

onMounted(async () => {
	await useCheckDisableMouseover()

	document.querySelector('body').addEventListener('click', handleClick)
	document.querySelector('body').addEventListener('auxclick', handleAuxClick)

	checkUpdates()
})

onUnmounted(async () => {
	document.querySelector('body').removeEventListener('click', handleClick)
	document.querySelector('body').removeEventListener('auxclick', handleAuxClick)

	await unlistenUpdateDownload?.()
})

const { formatMessage } = useVIntl()
const messages = defineMessages({
	updateInstalledToastTitle: {
		id: 'app.update.complete-toast.title',
		defaultMessage: 'Version {version} was successfully installed!',
	},
	updateInstalledToastText: {
		id: 'app.update.complete-toast.text',
		defaultMessage: 'Click here to view the changelog.',
	},
	reloadToUpdate: {
		id: 'app.update.reload-to-update',
		defaultMessage: 'Reload to install update',
	},
	downloadUpdate: {
		id: 'app.update.download-update',
		defaultMessage: 'Download update',
	},
	downloadingUpdate: {
		id: 'app.update.downloading-update',
		defaultMessage: 'Downloading update ({percent}%)',
	},
	authUnreachableHeader: {
		id: 'app.auth-servers.unreachable.header',
		defaultMessage: 'Cannot reach authentication servers',
	},
	authUnreachableBody: {
		id: 'app.auth-servers.unreachable.body',
		defaultMessage:
			'Minecraft authentication servers may be down right now. Check your internet connection and try again later.',
	},
})

async function setupApp() {
	const {
		native_decorations,
		theme,
		locale,
		telemetry,
		collapsed_navigation,
		advanced_rendering,
		onboarded,
		default_page,
		toggle_sidebar,
		developer_mode,
		feature_flags,
		pending_update_toast_for_version,
	} = await getSettings()

	// Initialize locale from saved settings
	if (locale) {
		i18n.global.locale.value = locale
	}

	if (default_page === 'Library') {
		await router.push('/library')
	}

	os.value = await getOS()
	const dev = await isDev()
	const version = await getVersion()
	showOnboarding.value = !onboarded

	nativeDecorations.value = native_decorations
	if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

	themeStore.setThemeState(theme)
	themeStore.collapsedNavigation = collapsed_navigation
	themeStore.advancedRendering = advanced_rendering
	themeStore.toggleSidebar = toggle_sidebar
	themeStore.devMode = developer_mode
	themeStore.featureFlags = feature_flags
	stateInitialized.value = true

	isMaximized.value = await getCurrentWindow().isMaximized()

	await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	initAnalytics()
	if (!telemetry) {
		optOutAnalytics()
	}
	if (dev) debugAnalytics()
	trackEvent('Launched', { version, dev, onboarded })

	if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

	const osType = await type()
	if (osType === 'macos') {
		document.getElementsByTagName('html')[0].classList.add('mac')
	} else {
		document.getElementsByTagName('html')[0].classList.add('windows')
	}

	await warning_listener((e) =>
		addNotification({
			title: 'Warning',
			text: e.message,
			type: 'warn',
		}),
	)

	useFetch(
		`https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
		'criticalAnnouncements',
		true,
	)
		.then((response) => response.json())
		.then((res) => {
			if (res && res.header && res.body) {
				criticalErrorMessage.value = res
			}
		})
		.catch(() => {
			console.log(
				`No critical announcement found at https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
			)
		})

	useFetch(`https://modrinth.com/news/feed/articles.json`, 'news', true)
		.then((response) => response.json())
		.then((res) => {
			if (res && res.articles) {
				// Format expected by NewsArticleCard component.
				news.value = res.articles
					.map((article) => ({
						...article,
						path: article.link,
						thumbnail: article.thumbnail,
						title: article.title,
						summary: article.summary,
						date: article.date,
					}))
					.slice(0, 4)
			}
		})

	get_opening_command().then(handleCommand)
	fetchCredentials()

	try {
		const skins = (await get_available_skins()) ?? []
		const capes = (await get_available_capes()) ?? []
		generateSkinPreviews(skins, capes)
	} catch (error) {
		console.warn('Failed to generate skin previews in app setup.', error)
	}

	if (pending_update_toast_for_version !== null) {
		const settings = await getSettings()
		settings.pending_update_toast_for_version = null
		await setSettings(settings)
	}

	if (osType === 'windows') {
		await processPendingSurveys()
	} else {
		console.info('Skipping user surveys on non-Windows platforms')
	}
}

const stateFailed = ref(false)
initialize_state()
	.then(() => {
		setupApp().catch((err) => {
			stateFailed.value = true
			console.error(err)
			error.showError(err, null, false, 'state_init')
		})
	})
	.catch((err) => {
		stateFailed.value = true
		console.error('Failed to initialize app', err)
		error.showError(err, null, false, 'state_init')
	})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}

const router = useRouter()
router.afterEach((to, from, failure) => {
	trackEvent('PageView', {
		path: to.path,
		fromPath: from.path,
		failed: failure,
	})
})
const route = useRoute()

const loading = useLoading()
loading.setEnabled(false)

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

const credentials = ref()

const modrinthLoginFlowWaitModal = ref()

async function fetchCredentials() {
	const creds = await getCreds().catch(handleError)
	if (creds && creds.user_id) {
		creds.user = await get_user(creds.user_id).catch(handleError)
	}
	credentials.value = creds ?? null
}

async function signIn() {
	modrinthLoginFlowWaitModal.value.show()

	try {
		await login()
		await fetchCredentials()
	} catch (error) {
		if (
			typeof error === 'object' &&
			typeof error['message'] === 'string' &&
			error.message.includes('Login canceled')
		) {
			// Not really an error due to being a result of user interaction, show nothing
		} else {
			handleError(error)
		}
	} finally {
		modrinthLoginFlowWaitModal.value.hide()
	}
}

async function logOut() {
	await logout().catch(handleError)
	await fetchCredentials()
}

const MIDAS_BITFLAG = 1 << 0
const hasPlus = computed(
	() =>
		credentials.value &&
		credentials.value.user &&
		(credentials.value.user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG,
)

const sidebarToggled = ref(true)

themeStore.$subscribe(() => {
	sidebarToggled.value = !themeStore.toggleSidebar
})

const forceSidebar = computed(
	() => route.path.startsWith('/browse') || route.path.startsWith('/project'),
)
const sidebarVisible = computed(() => sidebarToggled.value || forceSidebar.value)
const showAd = computed(
	() => sidebarVisible.value && !hasPlus.value && credentials.value !== undefined,
)

watch(showAd, () => {
	if (!showAd.value) {
		hide_ads_window(true)
	} else {
		init_ads_window(true)
	}
})

onMounted(() => {
	invoke('show_window')

	error.setErrorModal(errorModal.value)

	install.setIncompatibilityWarningModal(incompatibilityWarningModal)
	install.setInstallConfirmModal(installConfirmModal)
	install.setModInstallModal(modInstallModal)
})

const accounts = ref(null)
provide('accountsCard', accounts)

command_listener(handleCommand)
async function handleCommand(e) {
	if (!e) return

	if (e.event === 'RunMRPack') {
		// RunMRPack should directly install a local mrpack given a path
		if (e.path.endsWith('.mrpack')) {
			await create_profile_and_install_from_file(e.path).catch(handleError)
			trackEvent('InstanceCreate', {
				source: 'CreationModalFileDrop',
			})
		}
	} else {
		// Other commands are URL-based (deep linking)
		urlModal.value.show(e)
	}
}

const appUpdateDownload = {
	progress: ref(0),
	version: ref(),
}
let unlistenUpdateDownload

const downloadProgress = computed(() => appUpdateDownload.progress.value)
const downloadPercent = computed(() => Math.trunc(appUpdateDownload.progress.value * 100))

const metered = ref(true)
const finishedDownloading = ref(false)
const restarting = ref(false)
const updateToastDismissed = ref(false)
const availableUpdate = ref(null)
const updateSize = ref(null)
async function checkUpdates() {
	if (!(await areUpdatesEnabled())) {
		console.log('Skipping update check as updates are disabled in this build or environment')
		return
	}

	async function performCheck() {
		const update = await invoke('plugin:updater|check')
		const isExistingUpdate = update.version === availableUpdate.value?.version

		if (!update) {
			console.log('No update available')
			return
		}

		if (isExistingUpdate) {
			console.log('Update is already known')
			return
		}

		appUpdateDownload.progress.value = 0
		finishedDownloading.value = false
		updateToastDismissed.value = false

		console.log(`Update ${update.version} is available.`)

		metered.value = await isNetworkMetered()
		if (!metered.value) {
			console.log('Starting download of update')
			downloadUpdate(update)
		} else {
			console.log(`Metered connection detected, not auto-downloading update.`)
		}

		getUpdateSize(update.rid).then((size) => (updateSize.value = size))

		availableUpdate.value = update
	}

	await performCheck()
	setTimeout(
		() => {
			checkUpdates()
		},
		5 /* min */ * 60 /* sec */ * 1000 /* ms */,
	)
}

async function showUpdateToast() {
	updateToastDismissed.value = false
}

async function downloadAvailableUpdate() {
	return downloadUpdate(availableUpdate.value)
}

async function downloadUpdate(versionToDownload) {
	if (!versionToDownload) {
		handleError(`Failed to download update: no version available`)
	}

	if (appUpdateDownload.progress.value !== 0) {
		console.error(`Update ${versionToDownload.version} already downloading`)
		return
	}

	console.log(`Downloading update ${versionToDownload.version}`)

	try {
		enqueueUpdateForInstallation(versionToDownload.rid).then(() => {
			finishedDownloading.value = true
			unlistenUpdateDownload?.().then(() => {
				unlistenUpdateDownload = null
			})
			console.log('Finished downloading!')
		})
		unlistenUpdateDownload = await subscribeToDownloadProgress(
			appUpdateDownload,
			versionToDownload.version,
		)
	} catch (e) {
		handleError(e)
	}
}

async function installUpdate() {
	restarting.value = true
	setTimeout(async () => {
		await handleClose()
	}, 250)
}

function handleClick(e) {
	let target = e.target
	while (target != null) {
		if (target.matches('a')) {
			if (
				target.href &&
				['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
				!target.classList.contains('router-link-active') &&
				!target.href.startsWith('http://localhost') &&
				!target.href.startsWith('https://tauri.localhost') &&
				!target.href.startsWith('http://tauri.localhost')
			) {
				openUrl(target.href)
			}
			e.preventDefault()
			break
		}
		target = target.parentElement
	}
}

function handleAuxClick(e) {
	// disables middle click -> new tab
	if (e.button === 1) {
		e.preventDefault()
		// instead do a left click
		const event = new MouseEvent('click', {
			view: window,
			bubbles: true,
			cancelable: true,
		})
		e.target.dispatchEvent(event)
	}
}

function cleanupOldSurveyDisplayData() {
	const threeWeeksAgo = new Date()
	threeWeeksAgo.setDate(threeWeeksAgo.getDate() - 21)

	for (let i = 0; i < localStorage.length; i++) {
		const key = localStorage.key(i)

		if (key.startsWith('survey-') && key.endsWith('-display')) {
			const dateValue = new Date(localStorage.getItem(key))
			if (dateValue < threeWeeksAgo) {
				localStorage.removeItem(key)
			}
		}
	}
}

async function openSurvey() {
	if (!availableSurvey.value) {
		console.error('No survey to open')
		return
	}

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const formId = availableSurvey.value.tally_id

	const popupOptions = {
		layout: 'modal',
		width: 700,
		autoClose: 2000,
		hideTitle: true,
		hiddenFields: {
			user_id: userId,
		},
		onOpen: () => console.info('Opened user survey'),
		onClose: () => {
			console.info('Closed user survey')
			show_ads_window()
		},
		onSubmit: () => console.info('Active user survey submitted'),
	}

	try {
		hide_ads_window()
		if (window.Tally?.openPopup) {
			console.info(`Opening Tally popup for user survey (form ID: ${formId})`)
			dismissSurvey()
			window.Tally.openPopup(formId, popupOptions)
		} else {
			console.warn('Tally script not yet loaded')
			show_ads_window()
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
		show_ads_window()
	}

	console.info(`Found user survey to show with tally_id: ${formId}`)
	window.Tally.openPopup(formId, popupOptions)
}

function dismissSurvey() {
	localStorage.setItem(`survey-${availableSurvey.value.id}-display`, new Date())
	availableSurvey.value = undefined
}

async function processPendingSurveys() {
	function isWithinLastTwoWeeks(date) {
		const twoWeeksAgo = new Date()
		twoWeeksAgo.setDate(twoWeeksAgo.getDate() - 14)
		return date >= twoWeeksAgo
	}

	cleanupOldSurveyDisplayData()

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const instances = await list().catch(handleError)
	const isActivePlayer =
		instances.findIndex(
			(instance) =>
				isWithinLastTwoWeeks(instance.last_played) && !isWithinLastTwoWeeks(instance.created),
		) >= 0

	let surveys = []
	try {
		surveys = await $fetch('https://api.modrinth.com/v2/surveys')
	} catch (e) {
		console.error('Error fetching surveys:', e)
	}

	const surveyToShow = surveys.find(
		(survey) =>
			!!(
				localStorage.getItem(`survey-${survey.id}-display`) === null &&
				survey.type === 'tally_app' &&
				((survey.condition === 'active_player' && isActivePlayer) ||
					(survey.assigned_users?.includes(userId) && !survey.dismissed_users?.includes(userId)))
			),
	)

	if (surveyToShow) {
		availableSurvey.value = surveyToShow
	} else {
		console.info('No user survey to show')
	}
}

provideAppUpdateDownloadProgress(appUpdateDownload)
</script>

<template>
	<SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
	<div id="teleports"></div>
	<div
		v-if="stateInitialized"
		class="app-grid-layout experimental-styles-within relative"
		:class="{ 'disable-advanced-rendering': !themeStore.advancedRendering }"
	>
		<Suspense>
			<Transition name="toast">
				<UpdateToast
					v-if="
						!!availableUpdate &&
						!updateToastDismissed &&
						!restarting &&
						(finishedDownloading || metered)
					"
					:version="availableUpdate.version"
					:size="updateSize"
					:metered="metered"
					@close="updateToastDismissed = true"
					@restart="installUpdate"
					@download="downloadAvailableUpdate"
				/>
			</Transition>
		</Suspense>
		<Transition name="fade">
			<div
				v-if="restarting"
				data-tauri-drag-region
				class="inset-0 fixed bg-black/80 backdrop-blur z-[200] flex items-center justify-center"
			>
				<span
					data-tauri-drag-region
					class="flex items-center gap-4 text-contrast font-semibold text-xl select-none cursor-default"
				>
					<RefreshCwIcon data-tauri-drag-region class="animate-spin w-6 h-6" />
					Restarting...
				</span>
			</div>
		</Transition>
		<Suspense>
			<AppSettingsModal ref="settingsModal" />
		</Suspense>
		<Suspense>
			<AuthGrantFlowWaitModal ref="modrinthLoginFlowWaitModal" @flow-cancel="cancelLogin" />
		</Suspense>
		<Suspense>
			<InstanceCreationModal ref="installationModal" />
		</Suspense>
		<div
			class="app-grid-navbar bg-bg-raised flex flex-col p-[0.5rem] pt-0 gap-[0.5rem] w-[--left-bar-width]"
		>
			<NavButton v-tooltip.right="'Home'" to="/">
				<HomeIcon />
			</NavButton>
			<NavButton v-if="themeStore.featureFlags.worlds_tab" v-tooltip.right="'Worlds'" to="/worlds">
				<WorldIcon />
			</NavButton>
			<NavButton
				v-if="themeStore.featureFlags.servers_in_app"
				v-tooltip.right="'Servers'"
				to="/hosting/manage"
			>
				<ServerIcon />
			</NavButton>
			<NavButton
				v-tooltip.right="'Discover content'"
				to="/browse/modpack"
				:is-primary="() => route.path.startsWith('/browse') && !route.query.i"
				:is-subpage="(route) => route.path.startsWith('/project') && !route.query.i"
			>
				<CompassIcon />
			</NavButton>
			<NavButton v-tooltip.right="'Skins (Beta)'" to="/skins">
				<ChangeSkinIcon />
			</NavButton>
			<NavButton
				v-tooltip.right="'Library'"
				to="/library"
				:is-subpage="
					() =>
						route.path.startsWith('/instance') ||
						((route.path.startsWith('/browse') || route.path.startsWith('/project')) &&
							route.query.i)
				"
			>
				<LibraryIcon />
			</NavButton>
			<div class="h-px w-6 mx-auto my-2 bg-surface-5"></div>
			<suspense>
				<QuickInstanceSwitcher />
			</suspense>
			<NavButton
				v-tooltip.right="'Create new instance'"
				:to="() => $refs.installationModal.show()"
				:disabled="offline"
			>
				<PlusIcon />
			</NavButton>
			<div class="flex flex-grow"></div>
			<Transition name="nav-button-animated">
				<div
					v-if="
						availableUpdate &&
						updateToastDismissed &&
						!restarting &&
						(finishedDownloading || metered)
					"
				>
					<NavButton
						v-tooltip.right="
							formatMessage(
								finishedDownloading
									? messages.reloadToUpdate
									: downloadProgress === 0
										? messages.downloadUpdate
										: messages.downloadingUpdate,
								{
									percent: downloadPercent,
								},
							)
						"
						:to="
							finishedDownloading
								? installUpdate
								: downloadProgress > 0 && downloadProgress < 1
									? showUpdateToast
									: downloadAvailableUpdate
						"
					>
						<ProgressSpinner
							v-if="downloadProgress > 0 && downloadProgress < 1"
							class="text-brand"
							:progress="downloadProgress"
						/>
						<RefreshCwIcon v-else-if="finishedDownloading" class="text-brand" />
						<DownloadIcon v-else class="text-brand" />
					</NavButton>
				</div>
			</Transition>
			<NavButton
				v-tooltip.right="formatMessage(commonMessages.settingsLabel)"
				:to="() => $refs.settingsModal.show()"
			>
				<SettingsIcon />
			</NavButton>
			<OverflowMenu
				v-if="credentials"
				v-tooltip.right="`Modrinth account`"
				class="w-12 h-12 text-primary rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast border-0 cursor-pointer"
				:options="[
					{
						id: 'view-profile',
						action: () => openUrl('https://modrinth.com/user/' + credentials.user.username),
					},
					{
						id: 'sign-out',
						action: () => logOut(),
						color: 'danger',
					},
				]"
				placement="right-end"
			>
				<Avatar :src="credentials.user.avatar_url" alt="" size="32px" circle />
				<template #view-profile>
					<UserIcon />
					<span class="inline-flex items-center gap-1">
						Signed in as
						<span class="inline-flex items-center gap-1 text-contrast font-semibold">
							<Avatar :src="credentials.user.avatar_url" alt="" size="20px" circle />
							{{ credentials.user.username }}
						</span>
					</span>
					<ExternalIcon />
				</template>
				<template #sign-out> <LogOutIcon /> Sign out </template>
			</OverflowMenu>
			<NavButton v-else v-tooltip.right="'Sign in to a Modrinth account'" :to="() => signIn()">
				<LogInIcon class="text-brand" />
			</NavButton>
		</div>
		<div data-tauri-drag-region class="app-grid-statusbar bg-bg-raised h-[--top-bar-height] flex">
			<div data-tauri-drag-region class="flex p-3">
				<ModrinthAppLogo class="h-full w-auto text-contrast pointer-events-none" />
				<div data-tauri-drag-region class="flex items-center gap-1 ml-3">
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-full flex items-center justify-center w-6 h-6 hover:brightness-75 transition-all"
						@click="router.back()"
					>
						<LeftArrowIcon />
					</button>
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-full flex items-center justify-center w-6 h-6 hover:brightness-75 transition-all"
						@click="router.forward()"
					>
						<RightArrowIcon />
					</button>
				</div>
				<Breadcrumbs class="pt-[2px]" />
			</div>
			<section data-tauri-drag-region class="flex ml-auto items-center">
				<ButtonStyled
					v-if="!forceSidebar && themeStore.toggleSidebar"
					:type="sidebarToggled ? 'standard' : 'transparent'"
					circular
				>
					<button
						class="mr-3 transition-transform"
						:class="{ 'rotate-180': !sidebarToggled }"
						@click="sidebarToggled = !sidebarToggled"
					>
						<RightArrowIcon />
					</button>
				</ButtonStyled>
				<div class="flex mr-3">
					<Suspense>
						<RunningAppBar />
					</Suspense>
				</div>
				<section v-if="!nativeDecorations" class="window-controls" data-tauri-drag-region-exclude>
					<Button class="titlebar-button" icon-only @click="() => getCurrentWindow().minimize()">
						<MinimizeIcon />
					</Button>
					<Button
						class="titlebar-button"
						icon-only
						@click="() => getCurrentWindow().toggleMaximize()"
					>
						<RestoreIcon v-if="isMaximized" />
						<MaximizeIcon v-else />
					</Button>
					<Button class="titlebar-button close" icon-only @click="handleClose">
						<XIcon />
					</Button>
				</section>
			</section>
		</div>
	</div>
	<div
		v-if="stateInitialized"
		class="app-contents experimental-styles-within"
		:class="{
			'sidebar-enabled': sidebarVisible,
			'disable-advanced-rendering': !themeStore.advancedRendering,
		}"
	>
		<div class="app-viewport flex-grow router-view">
			<transition name="popup-survey">
				<div
					v-if="availableSurvey"
					class="w-[400px] z-20 fixed -bottom-12 pb-16 right-[--right-bar-width] mr-4 rounded-t-2xl card-shadow bg-bg-raised border-divider border-[1px] border-solid border-b-0 p-4"
				>
					<h2 class="text-lg font-extrabold mt-0 mb-2">Hey there Modrinth user!</h2>
					<p class="m-0 leading-tight">
						Would you mind answering a few questions about your experience with Modrinth App?
					</p>
					<p class="mt-3 mb-4 leading-tight">
						This feedback will go directly to the Modrinth team and help guide future updates!
					</p>
					<div class="flex gap-2">
						<ButtonStyled color="brand">
							<button @click="openSurvey"><NotepadTextIcon /> Take survey</button>
						</ButtonStyled>
						<ButtonStyled>
							<button @click="dismissSurvey"><XIcon /> No thanks</button>
						</ButtonStyled>
					</div>
				</div>
			</transition>
			<div
				class="loading-indicator-container h-8 fixed z-50"
				:style="{
					top: 'calc(var(--top-bar-height))',
					left: 'calc(var(--left-bar-width))',
					width: 'calc(100% - var(--left-bar-width) - var(--right-bar-width))',
				}"
			>
				<ModrinthLoadingIndicator />
			</div>
			<div
				v-if="themeStore.featureFlags.page_path"
				class="absolute bottom-0 left-0 m-2 bg-tooltip-bg text-tooltip-text font-semibold rounded-full px-2 py-1 text-xs z-50"
			>
				{{ route.fullPath }}
			</div>
			<div
				id="background-teleport-target"
				class="absolute h-full -z-10 rounded-tl-[--radius-xl] overflow-hidden"
				:style="{
					width: 'calc(100% - var(--right-bar-width))',
				}"
			></div>
			<Admonition
				v-if="criticalErrorMessage"
				type="critical"
				:header="criticalErrorMessage.header"
				class="m-6 mb-0"
			>
				<div
					class="markdown-body text-primary"
					v-html="renderString(criticalErrorMessage.body ?? '')"
				></div>
			</Admonition>
			<Admonition
				v-if="authUnreachable"
				type="warning"
				:header="formatMessage(messages.authUnreachableHeader)"
				class="m-6 mb-0"
			>
				{{ formatMessage(messages.authUnreachableBody) }}
			</Admonition>
			<RouterView v-slot="{ Component }">
				<template v-if="Component">
					<Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
						<component :is="Component"></component>
					</Suspense>
				</template>
			</RouterView>
		</div>
		<div
			class="app-sidebar mt-px shrink-0 flex flex-col border-0 border-l-[1px] border-[--brand-gradient-border] border-solid overflow-auto"
			:class="{ 'has-plus': hasPlus }"
		>
			<div
				class="app-sidebar-scrollable flex-grow shrink overflow-y-auto relative"
				:class="{ 'pb-12': !hasPlus }"
			>
				<div id="sidebar-teleport-target" class="sidebar-teleport-content"></div>
				<div class="sidebar-default-content" :class="{ 'sidebar-enabled': sidebarVisible }">
					<div
						class="p-4 pr-1 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					>
						<h3 class="text-base text-primary font-medium m-0">Playing as</h3>
						<suspense>
							<AccountsCard ref="accounts" mode="small" />
						</suspense>
					</div>
					<div class="py-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid">
						<suspense>
							<FriendsList
								:credentials="credentials"
								:sign-in="() => signIn()"
								:refresh-credentials="fetchCredentials"
							/>
						</suspense>
					</div>
					<div v-if="news && news.length > 0" class="p-4 pr-1 flex flex-col items-center">
						<h3 class="text-base mb-4 text-primary font-medium m-0 text-left w-full">News</h3>
						<div class="space-y-4 flex flex-col items-center w-full">
							<NewsArticleCard
								v-for="(item, index) in news"
								:key="`news-${index}`"
								:article="item"
							/>
							<ButtonStyled color="brand" size="large">
								<a href="https://modrinth.com/news" target="_blank" class="my-4">
									<NewspaperIcon /> View all news
								</a>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>
			<template v-if="showAd">
				<a
					href="https://modrinth.plus?app"
					class="absolute bottom-[250px] w-full flex justify-center items-center gap-1 px-4 py-3 text-purple font-medium hover:underline z-10"
					target="_blank"
				>
					<ArrowBigUpDashIcon class="text-2xl" /> Upgrade to Modrinth+
				</a>
				<PromotionWrapper />
			</template>
		</div>
	</div>
	<URLConfirmModal ref="urlModal" />
	<NotificationPanel has-sidebar />
	<ErrorModal ref="errorModal" />
	<ModInstallModal ref="modInstallModal" />
	<IncompatibilityWarningModal ref="incompatibilityWarningModal" />
	<InstallConfirmModal ref="installConfirmModal" />
</template>

<style lang="scss" scoped>
.window-controls {
	z-index: 20;
	display: none;
	flex-direction: row;
	align-items: center;

	.titlebar-button {
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all ease-in-out 0.1s;
		background-color: transparent;
		color: var(--color-base);
		height: 100%;
		width: 3rem;
		position: relative;
		box-shadow: none;

		&:last-child {
			padding-right: 0.75rem;
			width: 3.75rem;
		}

		svg {
			width: 1.25rem;
			height: 1.25rem;
		}

		&::before {
			content: '';
			border-radius: 999999px;
			width: 3rem;
			height: 3rem;
			aspect-ratio: 1 / 1;
			margin-block: auto;
			position: absolute;
			background-color: transparent;
			scale: 0.9;
			transition: all ease-in-out 0.2s;
			z-index: -1;
		}

		&.close {
			&:hover,
			&:active {
				color: var(--color-accent-contrast);

				&::before {
					background-color: var(--color-red);
				}
			}
		}

		&:hover,
		&:active {
			color: var(--color-contrast);

			&::before {
				background-color: var(--color-button-bg);
				scale: 1;
			}
		}
	}
}

.app-grid-layout,
.app-contents {
	--top-bar-height: 3rem;
	--left-bar-width: 4rem;
	--right-bar-width: 300px;
}

.app-grid-layout {
	display: grid;
	grid-template: 'status status' 'nav dummy';
	grid-template-columns: auto 1fr;
	grid-template-rows: auto 1fr;
	position: relative;
	//z-index: 0;
	background-color: var(--color-raised-bg);
	height: 100vh;
}

.app-grid-navbar {
	grid-area: nav;
}

.app-grid-statusbar {
	grid-area: status;
}

[data-tauri-drag-region-exclude] {
	-webkit-app-region: no-drag;
}

.app-contents {
	position: absolute;
	z-index: 1;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: 0;
	bottom: 0;
	height: calc(100vh - var(--top-bar-height));
	background-color: var(--color-bg);
	border-top-left-radius: var(--radius-xl);

	display: grid;
	grid-template-columns: 1fr 0px;
	// transition: grid-template-columns 0.4s ease-in-out;

	&.sidebar-enabled {
		grid-template-columns: 1fr 300px;
	}
}

.loading-indicator-container {
	border-top-left-radius: var(--radius-xl);
	overflow: hidden;
}

.app-sidebar {
	overflow: visible;
	width: 300px;
	position: relative;
	height: calc(100vh - var(--top-bar-height));
	background: var(--brand-gradient-bg);

	--color-button-bg: var(--brand-gradient-button);
	--color-button-bg-hover: var(--brand-gradient-border);
	--color-divider: var(--brand-gradient-border);
	--color-divider-dark: var(--brand-gradient-border);
}

.app-sidebar::after {
	content: '';
	position: absolute;
	bottom: 250px;
	left: 0;
	right: 0;
	height: 5rem;
	background: var(--brand-gradient-fade-out-color);
	pointer-events: none;
}

.app-sidebar.has-plus::after {
	display: none;
}

.disable-advanced-rendering {
	.app-sidebar::before {
		box-shadow: none;
	}

	&.app-contents::before {
		box-shadow: none;
	}
}

.app-sidebar::before {
	content: '';
	box-shadow: -15px 0 15px -15px rgba(0, 0, 0, 0.1) inset;
	top: 0;
	bottom: 0;
	left: -2rem;
	width: 2rem;
	position: absolute;
	pointer-events: none;
}

.app-viewport {
	flex-grow: 1;
	height: 100%;
	overflow: auto;
	overflow-x: hidden;
}

.app-contents::before {
	z-index: 1;
	content: '';
	position: fixed;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: calc(-1 * var(--left-bar-width));
	bottom: calc(-1 * var(--left-bar-width));
	border-radius: var(--radius-xl);
	box-shadow: 1px 1px 15px rgba(0, 0, 0, 0.1) inset;
	border-color: var(--surface-5);
	border-width: 1px;
	border-style: solid;
	pointer-events: none;
}

.sidebar-teleport-content {
	display: contents;
}

.sidebar-default-content {
	display: none;
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

.popup-survey-enter-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.51, 1.08, 0.35, 1.15);
	transform-origin: top center;
}

.popup-survey-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.68, -0.17, 0.23, 0.11);
	transform-origin: top center;
}

.popup-survey-enter-from,
.popup-survey-leave-to {
	opacity: 0;
	transform: translateY(10rem) scale(0.8) scaleY(1.6);
}

.toast-enter-active {
	transition: opacity 0.25s linear;
}

.toast-enter-from,
.toast-leave-to {
	opacity: 0;
}

@media (prefers-reduced-motion: no-preference) {
	.toast-enter-active,
	.nav-button-animated-enter-active {
		transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
	}

	.toast-leave-active,
	.nav-button-animated-leave-active {
		transition: all 0.25s ease;
	}

	.toast-enter-from {
		scale: 0.5;
		translate: 0 -10rem;
		opacity: 0;
	}

	.toast-leave-to {
		scale: 0.96;
		translate: 20rem 0;
		opacity: 0;
	}

	.nav-button-animated-enter-active {
		position: relative;
	}

	.nav-button-animated-enter-active::before {
		content: '';
		inset: 0;
		border-radius: 100vw;
		background-color: var(--color-brand-highlight);
		position: absolute;
		animation: pop 0.5s ease-in forwards;
		opacity: 0;
	}

	@keyframes pop {
		0% {
			scale: 0.5;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			scale: 1.5;
		}
	}

	.nav-button-animated-enter-from {
		scale: 0.5;
		translate: -2rem 0;
		opacity: 0;
	}

	.nav-button-animated-leave-to {
		scale: 0.75;
		opacity: 0;
	}

	.fade-enter-active {
		transition: 0.25s ease-in-out;
	}

	.fade-enter-from {
		opacity: 0;
	}
}
</style>
<style>
.mac {
	.app-grid-statusbar {
		padding-left: 5rem;
	}
}

.windows {
	.fake-appbar {
		height: 2.5rem !important;
	}

	.window-controls {
		display: flex !important;
	}

	.info-card {
		right: 8rem;
	}

	.profile-card {
		right: 8rem;
	}
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
