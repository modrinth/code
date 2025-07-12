import type { Nag } from '../types/nags'
import { coreNags } from './nags/core'
import { descriptionNags } from './nags/description'
import { linksNags } from './nags/links'

export default [...coreNags, ...linksNags, ...descriptionNags] as Nag[]
