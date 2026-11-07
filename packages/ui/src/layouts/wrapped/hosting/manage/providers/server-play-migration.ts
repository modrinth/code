import type { UploadProgress } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'

import type {
	ServerPlayMigrationKind,
	ServerPlayMigrationSource,
} from '#ui/components/servers/play-migration/types'
import { createContext } from '#ui/providers/create-context'

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
	/** Whether Archon flagged the server for migration. */
	tagged: ComputedRef<boolean>
	required: ComputedRef<boolean>
	/** `null` until the server's content, and how it was installed, are known. */
	kind: ComputedRef<ServerPlayMigrationKind | null>
	/** Whether the mrpack the server was installed from is still in the world, so it needn't be uploaded again. */
	hasUploadedMrpack: ComputedRef<boolean>
	step: Ref<ServerPlayMigrationStep>
	/** Progress of the pack being uploaded for the reinstall, `null` when nothing is uploading. */
	uploadProgress: Ref<UploadProgress | null>
	/** Whether a migration is running, including one started before this panel was mounted. */
	active: ComputedRef<boolean>
	modpackName: ComputedRef<string | null>
	prompt: (next: ServerPlayMigrationNext) => void
	/** Runs the migration, calling `onRequested` once the reinstall is accepted, after any upload. */
	start: (
		source: ServerPlayMigrationSource,
		next?: ServerPlayMigrationNext,
		onRequested?: () => void,
	) => Promise<boolean>
}

export const [injectServerPlayMigration, provideServerPlayMigration] =
	createContext<ServerPlayMigration>('[id].vue', 'serverPlayMigration')
