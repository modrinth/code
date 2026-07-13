import type { StageNodeBuilder } from '../types/node'
import { group } from '../types/node'
import categories from './stages/categories'
import description from './stages/description'
import metadata from './stages/metadata'
import gallery from './stages/gallery'
import license from './stages/license'
import links from './stages/links'
import permissions from './stages/permissions'
import postApproval from './stages/post-approval'
import reupload from './stages/reupload'
import ruleFollowing from './stages/rule-following'
import statusAlerts from './stages/status-alerts'
import summary from './stages/summary'
import titleSlug from './stages/title-slug'
import undefinedProject from './stages/undefined-project'
import versions from './stages/versions'

export const stages = [
	titleSlug,
	summary,
	description,
	links,
	license,
	categories,
	metadata,
	gallery,
	versions,
	reupload,
	permissions,
	ruleFollowing,
	statusAlerts,
	undefinedProject,
	postApproval,
] as ReadonlyArray<StageNodeBuilder>

export default group().children(...stages)
