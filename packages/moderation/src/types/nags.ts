import type { Labrinth } from '@modrinth/api-client'
import type { MessageDescriptor } from '@modrinth/ui'
import type { FunctionalComponent, SVGAttributes } from 'vue'

/**
 * Type which represents the status type of a nag.
 *
 * - `required` indicates that the nag must be addressed.
 * - `warning` indicates that the nag is important but not critical, and can be ignored. It is often used for issues that should be resolved but do not block project submission.
 * - `suggestion` indicates that the nag is a recommendation and can be ignored.
 */
export type NagStatus = 'required' | 'warning' | 'suggestion' | 'special-submit-action'

export type NagDestinationId =
	| 'description'
	| 'disclosures'
	| 'gallery'
	| 'general'
	| 'license'
	| 'links'
	| 'moderation'
	| 'permissions'
	| 'server'
	| 'tags'
	| 'versions'

/**
 * Data required to validate a project.
 */
export interface ProjectValidationContext {
	/**
	 * The project associated with the nag.
	 */
	project: Labrinth.Projects.v2.Project
	/**
	 * The project V3 associated with the nag.
	 */
	projectV3: Labrinth.Projects.v3.Project
	/**
	 * The versions associated with the project.
	 */
	versions: Labrinth.Versions.v3.Version[]

	tags: {
		categories?: Labrinth.Tags.v2.Category[]
		rejectedStatuses: string[]
	}
}

/**
 * Context required to render a nag and its navigation.
 */
export interface NagContext extends ProjectValidationContext {
	/**
	 * The current project member viewing the nag.
	 */
	currentMember?: Labrinth.Users.v2.User
	/**
	 * The current route in the application.
	 */
	currentRoute: string
}

/**
 * Interface representing a nag's link.
 */
export interface NagLink {
	/**
	 * A relative path to the nag's link, e.g. '/settings'.
	 */
	path: string
	/**
	 * The text to display for the nag's link.
	 */
	title: MessageDescriptor | string
	/**
	 * Whether to show the link in the current context.
	 */
	shouldShow: (context: NagContext) => boolean
}

/**
 * Interface representing a nag.
 */
export interface Nag {
	/**
	 * A unique identifier for the nag.
	 */
	id: string
	/**
	 * The title of the nag.
	 */
	title: MessageDescriptor | string
	/**
	 * A function that returns the description of the nag.
	 * It can accept a context to provide dynamic descriptions.
	 */
	description: MessageDescriptor | ((context: ProjectValidationContext) => string)
	/** Values used when formatting a message descriptor description. */
	values?: Record<string, string | number | boolean>
	/**
	 * The status of the nag, which can be 'required', 'warning', or 'suggestion'.
	 */
	status: NagStatus
	/**
	 * An optional icon for the nag, usually from `@modrinth/assets`.
	 * If not specified it will use the default icon associated with the nag status.
	 */
	icon?: FunctionalComponent<SVGAttributes>

	/**
	 * A function that determines whether the nag should be shown based on the context.
	 */
	shouldShow: (context: ProjectValidationContext) => boolean
	/**
	 * An optional link associated with the nag.
	 * If provided, it should be displayed alongside the nag.
	 */
	link?: NagLink
}
