import type { Stage } from '../types/stage'
import modpackPermissionsStage from './modpack-permissions-stage'
import categories from './stages/categories'
import description from './stages/description'
import environment from './stages/environment/environment'
import environmentMultiple from './stages/environment/environment-multiple'
import gallery from './stages/gallery'
import license from './stages/license'
import links from './stages/links'
import postApproval from './stages/post-approval'
import reupload from './stages/reupload'
import ruleFollowing from './stages/rule-following'
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
	environment,
	environmentMultiple,
	gallery,
	versions,
	reupload,
	ruleFollowing,
	modpackPermissionsStage,
	statusAlerts,
	undefinedProject,
	postApproval,
] as ReadonlyArray<Stage>
