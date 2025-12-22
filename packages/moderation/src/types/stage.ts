import type { Labrinth } from '@modrinth/api-client'
import type { FunctionalComponent, SVGAttributes } from 'vue'

import type { Action } from './actions'

/**
 * Represents a moderation stage with associated actions and optional navigation logic.
 */
export interface Stage {
	/**
	 * The title of the stage, displayed to the moderator.
	 */
	title: string

	/**
	 * An optional description or additional text for the stage.
	 */
	text?: (
		project: Labrinth.Projects.v2.Project,
		projectV3?: Labrinth.Projects.v3.Project,
	) => Promise<string>

	/**
	 * Optional id for the stage, used for identification in the checklist. Will be used in the stage list as well instead of the title.
	 */
	id?: string

	/**
	 * Optional icon for the stage, displayed in the stage list and next to the title.
	 */
	icon?: FunctionalComponent<SVGAttributes>

	/**
	 * URL to the guidance document for this stage.
	 */
	guidance_url: string

	/**
	 * An array of actions that can be taken in this stage.
	 */
	actions: Action[]

	/**
	 * Optional navigation path to redirect the moderator when this stage is shown.
	 *
	 * This is relative to the project page. For example, `/settings#side-types` would navigate to `https://modrinth.com/project/:id/settings#side-types`.
	 */
	navigate?: string

	/**
	 * A function that determines whether this stage should be shown for a given project.
	 *
	 * By default, it returns `true`, meaning the stage is always shown.
	 */
	shouldShow?: (
		project: Labrinth.Projects.v2.Project,
		projectV3?: Labrinth.Projects.v3.Project,
	) => boolean
}
