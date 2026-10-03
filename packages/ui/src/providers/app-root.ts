import type {MaybeElement} from "@vueuse/core";
import type {ComputedRef} from "vue";

import { createContext } from './create-context'

export const [injectAppRoot, provideAppRoot] = createContext<{root: ComputedRef<MaybeElement>}>(
	'root',
	'appRoot',
)
