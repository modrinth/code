import type { Stage } from '../types/stage'
import modpackPermissionsStage from './modpack-permissions-stage'
import categories from './stages/categories'
import description from './stages/description'
import gallery from './stages/gallery'
import license from './stages/license'
import links from './stages/links'
import postApproval from './stages/post-approval'
import reupload from './stages/reupload'
import ruleFollowing from './stages/rule-following'
import sideTypes from './stages/side-types'
import statusAlerts from './stages/status-alerts'
import summary from './stages/summary'
import titleSlug from './stages/title-slug'
import undefinedProject from './stages/undefined-project'
import versions from './stages/versions'

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
	postApproval,
] as ReadonlyArray<Stage>
