import type { Nag } from '../types/nags'
import { coreNags } from './nags/core'
import { linksNags } from './nags/links'
import { projectValidationNags } from './nags/project-validation'
import { serverProjectsNags } from './nags/server-projects'
import { tagsNags } from './nags/tags'

export default [
	...coreNags,
	...linksNags,
	...projectValidationNags,
	...tagsNags,
	...serverProjectsNags,
] as Nag[]
