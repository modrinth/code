import type { ComputedRef, Ref } from 'vue'

import { createContext } from './create-context'

export const SERVER_PLAY_MIGRATION_TAG = 'needs_server_play_reinstall_migration'

export type ServerPlayMigrationStep = 'idle' | 'migrating'

/** What happens once the migration succeeds: open the invite link modal, or run a callback. */
export type ServerPlayMigrationNext = 'invite' | (() => void)

/**
 * Servers whose content was installed before server play need a backup, reinstall and restart
 * before a shared instance can be created for them.
 */
export interface ServerPlayMigration {
	/** False while the server's content is still loading, so `required` can't be trusted yet. */
	ready: ComputedRef<boolean>
	/** Whether Archon flagged the server for migration, or the debug flag spoofs it. */
	tagged: ComputedRef<boolean>
	required: ComputedRef<boolean>
	step: Ref<ServerPlayMigrationStep>
	/** Whether a migration is running, including one started before this panel was mounted. */
	active: ComputedRef<boolean>
	modpackName: ComputedRef<string | null>
	prompt: (next: ServerPlayMigrationNext) => void
	start: (next?: ServerPlayMigrationNext) => Promise<boolean>
}

export const [injectServerPlayMigration, provideServerPlayMigration] =
	createContext<ServerPlayMigration>('[id].vue', 'serverPlayMigration')
