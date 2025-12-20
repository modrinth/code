import type { AbstractModrinthClient } from '../core/abstract-client'
import type { AbstractModule } from '../core/abstract-module'
import { ArchonBackupsV0Module } from './archon/backups/v0'
import { ArchonBackupsV1Module } from './archon/backups/v1'
import { ArchonServersV0Module } from './archon/servers/v0'
import { ArchonServersV1Module } from './archon/servers/v1'
import { ISO3166Module } from './iso3166'
import { KyrosFilesV0Module } from './kyros/files/v0'
import { LabrinthVersionsV3Module } from './labrinth'
import { LabrinthBillingInternalModule } from './labrinth/billing/internal'
import { LabrinthCollectionsModule } from './labrinth/collections'
import { LabrinthProjectsV2Module } from './labrinth/projects/v2'
import { LabrinthProjectsV3Module } from './labrinth/projects/v3'
import { LabrinthStateModule } from './labrinth/state'
import { LabrinthTechReviewInternalModule } from './labrinth/tech-review/internal'

type ModuleConstructor = new (client: AbstractModrinthClient) => AbstractModule

/**
 * To add a new module:
 * 1. Create your module class extending AbstractModule
 * 2. Add one line here: `<api>_<module>: YourModuleClass`
 *
 * TypeScript will automatically infer the client's field structure from this registry.
 *
 * TODO: Better way? Probably not
 */
export const MODULE_REGISTRY = {
	archon_backups_v0: ArchonBackupsV0Module,
	archon_backups_v1: ArchonBackupsV1Module,
	archon_servers_v0: ArchonServersV0Module,
	archon_servers_v1: ArchonServersV1Module,
	iso3166_data: ISO3166Module,
	kyros_files_v0: KyrosFilesV0Module,
	labrinth_billing_internal: LabrinthBillingInternalModule,
	labrinth_collections: LabrinthCollectionsModule,
	labrinth_projects_v2: LabrinthProjectsV2Module,
	labrinth_projects_v3: LabrinthProjectsV3Module,
	labrinth_state: LabrinthStateModule,
	labrinth_tech_review_internal: LabrinthTechReviewInternalModule,
	labrinth_versions_v3: LabrinthVersionsV3Module,
} as const satisfies Record<string, ModuleConstructor>

export type ModuleID = keyof typeof MODULE_REGISTRY

/**
 * Parse a module ID into [api, moduleName] tuple
 *
 * @param id - Module ID in format `<api>_<module>` (e.g., 'labrinth_projects_v2')
 * @returns Tuple of [api, moduleName] (e.g., ['labrinth', 'projects_v2'])
 * @throws Error if module ID doesn't match expected format
 */
export function parseModuleID(id: string): [string, string] {
	const parts = id.split('_')
	if (parts.length < 2) {
		throw new Error(
			`Invalid module ID "${id}". Expected format: <api>_<module> (e.g., "labrinth_projects_v2")`,
		)
	}
	const api = parts[0]
	const moduleName = parts.slice(1).join('_')
	return [api, moduleName]
}

/**
 * Build nested module structure from flat registry
 *
 * Transforms:
 * ```
 * { labrinth_projects_v2: Constructor, labrinth_users_v2: Constructor }
 * ```
 * Into:
 * ```
 * { labrinth: { projects_v2: Constructor, users_v2: Constructor } }
 * ```
 *
 * @returns Nested structure organized by API namespace
 */
export function buildModuleStructure(): Record<string, Record<string, ModuleConstructor>> {
	const structure: Record<string, Record<string, ModuleConstructor>> = {}

	for (const [id, constructor] of Object.entries(MODULE_REGISTRY)) {
		const [api, moduleName] = parseModuleID(id)

		if (!structure[api]) {
			structure[api] = {}
		}

		structure[api][moduleName] = constructor
	}

	return structure
}

/**
 * Extract API name from module ID
 * @example ParseAPI<'labrinth_projects_v2'> = 'labrinth'
 */
type ParseAPI<T extends string> = T extends `${infer API}_${string}` ? API : never

/**
 * Extract module name for a given API
 * @example ParseModule<'labrinth_projects_v2', 'labrinth'> = 'projects_v2'
 */
type ParseModule<T extends string, API extends string> = T extends `${API}_${infer Module}`
	? Module
	: never

/**
 * Group registry modules by API namespace
 *
 * Transforms flat registry into nested structure at the type level:
 * ```
 * { labrinth_projects_v2: ModuleClass } → { labrinth: { projects_v2: ModuleInstance } }
 * ```
 */
type GroupByAPI<Registry extends Record<string, ModuleConstructor>> = {
	[API in ParseAPI<keyof Registry & string>]: {
		[Module in ParseModule<keyof Registry & string, API>]: InstanceType<
			Registry[`${API}_${Module}`]
		>
	}
}

/**
 * Inferred client module structure
 **/
export type InferredClientModules = GroupByAPI<typeof MODULE_REGISTRY>
