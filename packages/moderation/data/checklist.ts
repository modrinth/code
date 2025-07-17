import type { Stage } from '../types/stage'
import modpackPermissionsStage from './modpack-permissions-stage'
import categories from './stages/categories'
import reupload from './stages/reupload'
import description from './stages/description'
import gallery from './stages/gallery'
import links from './stages/links'
import ruleFollowing from './stages/rule-following'
import sideTypes from './stages/side-types'
import summary from './stages/summary'
import titleSlug from './stages/title-slug'
import versions from './stages/versions'
import license from './stages/license'
import undefinedProject from './stages/undefined-project'
import statusAlerts from './stages/status-alerts'

export default [
  titleSlug,
  summary,
  description,
  links,
  license,
  categories,
  sideTypes,
  gallery,
  versions,
  reupload,
  ruleFollowing,
  modpackPermissionsStage,
  statusAlerts,
  undefinedProject,
] as ReadonlyArray<Stage>
