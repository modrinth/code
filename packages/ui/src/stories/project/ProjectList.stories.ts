import type { Labrinth } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectList from '../../components/project/ProjectList.vue'

const project: Labrinth.Projects.v3.Project = {
  id: 'example',
  slug: 'example-project',
  project_types: ['mod'],
  games: ['minecraft-java'],
  team_id: 'example-team',
  name: 'Example project',
  summary: 'Canonical v3 project state',
  description: '',
  published: '2026-01-01T00:00:00Z',
  updated: '2026-01-01T00:00:00Z',
  visibility: 'listed',
  in_review: false,
  review_outcome: 'approved',
  license: { id: 'MIT', name: 'MIT License' },
  downloads: 1234,
  followers: 100,
  categories: [],
  additional_categories: [],
  loaders: ['fabric'],
  mrpack_loaders: [],
  versions: [],
  link_urls: {},
  gallery: [],
  thread_id: 'example-thread',
  monetization_status: 'monetized',
  side_types_migration_review_status: 'reviewed',
}

const meta = {
  title: 'Project/ProjectList',
  component: ProjectList,
  args: {
    projects: [project],
    showStatus: true,
  },
} satisfies Meta<typeof ProjectList>

export default meta
type Story = StoryObj<typeof meta>

export const Approved: Story = {}
export const CanonicalStates: Story = {
  args: {
    projects: [
      project,
      {
        ...project,
        id: 'draft',
        name: 'Draft',
        review_outcome: 'never_reviewed',
      },
      { ...project, id: 'review', name: 'In review', in_review: true },
      {
        ...project,
        id: 'rejected',
        name: 'Rejected',
        review_outcome: 'rejected',
      },
      {
        ...project,
        id: 'withheld',
        name: 'Withheld',
        review_outcome: 'withheld',
      },
      { ...project, id: 'unlisted', name: 'Unlisted', visibility: 'unlisted' },
      { ...project, id: 'private', name: 'Private', visibility: 'private' },
    ],
  },
}
export const StatusHidden: Story = {
  args: { showStatus: false },
}
