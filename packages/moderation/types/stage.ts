import type { Project } from '@modrinth/utils'
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
  shouldShow?: ((project: Project) => boolean) | true
}
