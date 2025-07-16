import type { Stage } from '../types/stage'
import modpackPermissionsStage from './modpack-permissions-stage'
import categories from './stages/categories'
import copyright from './stages/copyright'
import description from './stages/description'
import gallery from './stages/gallery'
import links from './stages/links'
import ruleFollowing from './stages/rule-following'
import sideTypes from './stages/side-types'
import slug from './stages/slug'
import summary from './stages/summary'
import title from './stages/title'
import versions from './stages/versions'

export default [
  title,
  slug,
  summary,
  description,
  links,
  categories,
  sideTypes,
  gallery,
  versions,
  copyright,
  ruleFollowing,
  modpackPermissionsStage,
] as ReadonlyArray<Stage>
