import type { Labrinth } from '@modrinth/api-client'
import type { User, Version } from '@modrinth/utils'
import type { MessageDescriptor } from '@vintl/vintl'
import type { FunctionalComponent, SVGAttributes } from 'vue'

/**
 * Type which represents the status type of a nag.
 *
 * - `required` indicates that the nag must be addressed.
 * - `warning` indicates that the nag is important but not critical, and can be ignored. It is often used for issues that should be resolved but do not block project submission.
 * - `suggestion` indicates that the nag is a recommendation and can be ignored.
 */
export type NagStatus = 'required' | 'warning' | 'suggestion' | 'special-submit-action'

/**
 * Interface representing the context in which a nag is displayed.
 * It includes the project, versions, current member, all members, and the current route.
 * This context is used to determine whether a nag or it's link should be shown and how it should be presented.
 */
export interface NagContext {
	/**
	 * The project associated with the nag.
	 */
	project: Labrinth.Projects.v2.Project
	/**
	 * The versions associated with the project.
	 */
	versions: Version[]
	/**
	 * The current project member viewing the nag.
	 */
	currentMember: User
	/**
	 * The current route in the application.
	 */
	currentRoute: string
	/* eslint-disable @typescript-eslint/no-explicit-any */
	tags: any
	submitProject: (...any: any) => any
	/* eslint-enable @typescript-eslint/no-explicit-any */
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
	 * The status of the nag, which can be 'required', 'warning', or 'suggestion'.
	 */
	shouldShow?: (context: NagContext) => boolean
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
	description: MessageDescriptor | ((context: NagContext) => string)
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
	shouldShow: (context: NagContext) => boolean
	/**
	 * An optional link associated with the nag.
	 * If provided, it should be displayed alongside the nag.
	 */
	link?: NagLink
}
