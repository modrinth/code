import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { DisclosureFormState } from './types'

export const [injectDisclosureDraft, provideDisclosureDraft] =
	createContext<Ref<DisclosureFormState>>('DisclosureDraft')
