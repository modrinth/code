import { createContext } from './create-context'
import type {MaybeElement} from "@vueuse/core";
import type {ComputedRef} from "vue";

export const [injectAppRoot, provideAppRoot] = createContext<{root: ComputedRef<MaybeElement>}>(
	'root',
	'appRoot',
)
