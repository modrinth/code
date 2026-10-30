import type { CurseForgeSelection } from '#ui/components/flows/creation-flow-modal/curseforge'

/** How the server's content was installed, which decides what the migration reinstalls it from. */
export type ServerPlayMigrationKind = 'modrinth' | 'custom' | 'mrpack' | 'curseforge'

/** What the migration reinstalls the server's content from. `file: null` reuses the uploaded mrpack. */
export type ServerPlayMigrationSource =
	| { kind: 'restore' }
	| { kind: 'mrpack'; file: File | null }
	| { kind: 'curseforge'; selection: CurseForgeSelection }
