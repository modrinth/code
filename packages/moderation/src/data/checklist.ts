import type { ChecklistStage } from '../types/node'
import description from './stages/description'
import gallery from './stages/gallery'
import license from './stages/license'
import links from './stages/links'
import reupload from './stages/reupload'
import summary from './stages/summary'
import titleSlug from './stages/title-slug'
import versions from './stages/versions'

export default [titleSlug, summary, description, links, license, gallery, versions, reupload] as ReadonlyArray<ChecklistStage>
