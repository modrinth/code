import type { Ref } from 'vue'

import { createContext } from './create-context'

export interface ModalBehavior {
	noblur: Ref<boolean>
	onShow?: () => void
	onHide?: () => void
}

export const [injectModalBehavior, provideModalBehavior] = createContext<ModalBehavior>(
	'root',
	'modalBehavior',
)
