import type { Nag } from '../types/nags'
import { coreNags } from './nags/core'
import { descriptionNags } from './nags/description'
import { linksNags } from './nags/links'
import { tagsNags } from './nags/tags'

export default [...coreNags, ...linksNags, ...descriptionNags, ...tagsNags] as Nag[]
