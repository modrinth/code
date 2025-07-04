import type { Project } from '@modrinth/utils'
import type { Stage } from '../types/stage'

export default {
  id: 'modpack-permissions',
  title: 'Modpack Permissions',
  // Replace me please.
  guidance_url: 'https://docs.modrinth.com/moderation/modpack-permissions',
  shouldShow: (project: Project) => project.project_type === 'modpack',
  actions: [],
} as Stage
