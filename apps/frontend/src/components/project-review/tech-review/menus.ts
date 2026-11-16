import { createContext } from '@modrinth/ui'
import { inject, type InjectionKey, nextTick, onBeforeUnmount, type Ref } from 'vue'

import { createTechReviewMenuController } from './menu-controller'

export const folderMenuLockKey: InjectionKey<Ref<symbol | undefined>> = Symbol('FolderMenuLock')

export const registerFolderMenuChildKey: InjectionKey<
	(dismiss: (event: MouseEvent) => void) => () => void
> = Symbol('RegisterFolderMenuChild')

const [injectTechReviewMenus, provideMenus] =
	createContext<ReturnType<typeof createTechReviewMenuController>>('TechReviewMenus')

export function provideTechReviewMenus() {
	const controller = createTechReviewMenuController()
	provideMenus(controller)
	onBeforeUnmount(() => controller.dismiss())
}

export function useTechReviewMenu(options: {
	getTrigger: () => HTMLElement | null | undefined
	getPanel: () => HTMLElement | null | undefined
	hide: () => void
}) {
	const controller = injectTechReviewMenus()
	let registration: ReturnType<typeof controller.register> | undefined
	let releasePanel: (() => void) | undefined

	function stopPropagation(event: Event) {
		event.stopPropagation()
	}

	function syncPanel() {
		const panel = options.getPanel()
		if (!registration || !panel) return
		releasePanel?.()
		const previousLayer = panel.style.zIndex
		panel.style.zIndex = String(registration.zIndex)
		panel.addEventListener('pointerdown', stopPropagation)
		panel.addEventListener('click', stopPropagation)
		releasePanel = () => {
			panel.style.zIndex = previousLayer
			panel.removeEventListener('pointerdown', stopPropagation)
			panel.removeEventListener('click', stopPropagation)
		}
	}

	function close() {
		registration?.unregister()
		registration = undefined
		releasePanel?.()
		releasePanel = undefined
	}

	function open() {
		close()
		const trigger = options.getTrigger()
		if (!trigger) return
		registration = controller.register({
			document: trigger.ownerDocument,
			trigger,
			getPanel: () => options.getPanel() ?? null,
			hide: options.hide,
			onEscape: () => {
				const button = trigger.matches('button') ? trigger : trigger.querySelector('button')
				button?.focus()
				options.hide()
			},
		})
		syncPanel()
		void nextTick(syncPanel)
	}

	onBeforeUnmount(close)
	return { open, close }
}

export function useTechReviewOverflowMenu(
	menu: Ref<{ close: () => void } | undefined>,
	anchor: Ref<HTMLElement | undefined>,
) {
	const registerWithFolder = inject(registerFolderMenuChildKey, undefined)
	let unregisterFromFolder: (() => void) | undefined
	const coordination = useTechReviewMenu({
		getTrigger: () => anchor.value?.closest('button'),
		getPanel: () => {
			const button = anchor.value?.closest('button')
			const panelId = button?.getAttribute('aria-controls')
			return panelId ? button?.ownerDocument.getElementById(panelId) : null
		},
		hide: () => menu.value?.close(),
	})
	function close() {
		unregisterFromFolder?.()
		unregisterFromFolder = undefined
		coordination.close()
	}
	onBeforeUnmount(close)
	return {
		open: () => {
			coordination.open()
			unregisterFromFolder = registerWithFolder?.((event) => {
				if (!anchor.value?.closest('button')?.contains(event.target as Node)) menu.value?.close()
			})
		},
		close,
	}
}
