import { createContext } from '@modrinth/ui'

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
