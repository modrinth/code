import { createContext } from '@modrinth/ui'
import type { InjectionKey, Ref } from 'vue'

export const folderMenuLockKey: InjectionKey<Ref<symbol | undefined>> = Symbol('FolderMenuLock')

export const registerFolderMenuChildKey: InjectionKey<
	(dismiss: (event: MouseEvent) => void) => () => void
> = Symbol('RegisterFolderMenuChild')

type ReviewMenu = { hide: () => void }

const [injectTechReviewMenus, provideMenus] =
	createContext<(menu: ReviewMenu | undefined) => void>('TechReviewMenus')

export { injectTechReviewMenus }

export function provideTechReviewMenus() {
	let activeMenu: ReviewMenu | undefined
	return provideMenus((menu) => {
		if (!menu) return
		if (activeMenu !== menu) activeMenu?.hide()
		activeMenu = menu
	})
}
