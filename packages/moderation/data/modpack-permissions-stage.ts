import type { ModerationModpackPermissionApprovalType, Project } from '@modrinth/utils'
import type { Stage } from '../types/stage'
import { BoxIcon } from '@modrinth/assets'

export default {
  id: 'modpack-permissions',
  title: 'Modpack Permissions',
  icon: BoxIcon,
  // Replace me please.
  guidance_url: 'https://docs.modrinth.com/moderation/modpack-permissions',
  shouldShow: (project: Project) => project.project_type === 'modpack',
  actions: [],
} as Stage

export const finalPermissionMessages: Record<
  ModerationModpackPermissionApprovalType['id'],
  string | undefined
> = {
  yes: undefined,
  'with-attribution-and-source': undefined,
  'with-attribution': `The following content has attribution requirements, meaning that you must link back to the page where you originally found this content in your modpack description or version changelog (e.g. linking a mod's CurseForge page if you got it from CurseForge):`,
  no: 'The following content is not allowed in Modrinth modpacks due to licensing restrictions. Please contact the author(s) directly for permission or remove the content from your modpack:',
  'permanent-no': `The following content is not allowed in Modrinth modpacks, regardless of permission obtained. This may be because it breaks Modrinth's content rules or because the authors, upon being contacted for permission, have declined. Please remove the content from your modpack:`,
  unidentified: `The following content could not be identified. Please provide proof of its origin along with proof that you have permission to include it:`,
}
