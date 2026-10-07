import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { Skin } from '@/helpers/skins'

export interface MinecraftAccountsActions {
	readonly loginDisabled: boolean
	refreshValues: () => Promise<void>
	setEquippedSkin: (skin: Skin) => Promise<void>
	setLoginDisabled: (disabled: boolean) => void
	login: () => Promise<void>
}

export const [injectMinecraftAccounts, provideMinecraftAccounts] = createContext<
	Readonly<Ref<MinecraftAccountsActions | null>>
>('root', 'minecraftAccounts')
