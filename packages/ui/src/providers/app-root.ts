import type { ComputedRef } from 'vue'

import { createContext } from './create-context'

export const [injectAppRoot, provideAppRoot] = createContext<{
	root: ComputedRef<HTMLElement | SVGElement | Window | Document | null | undefined>
}>('root', 'appRoot')
