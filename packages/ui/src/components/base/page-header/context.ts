import type { Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

export interface PageHeaderContext {
	fadeItemCount: Ref<number>
}

export const [injectPageHeaderContext, providePageHeaderContext] = createContext<PageHeaderContext>(
	'PageHeader',
	'pageHeader',
)
