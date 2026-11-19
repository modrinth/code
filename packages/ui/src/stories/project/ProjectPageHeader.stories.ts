import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectPageHeader from '../../components/project/ProjectPageHeader.vue'

const meta = {
  title: 'Project/ProjectPageHeader',
  component: ProjectPageHeader,
  args: {
    project: {
      id: 'example',
      title: 'Example project',
      description: 'A project using canonical v3 review and visibility state.',
      status: 'approved',
      downloads: 1234,
      followers: 100,
      categories: [],
    },
    projectV3: {
      visibility: 'listed',
      in_review: false,
      review_outcome: 'approved',
      minecraft_server: null,
      minecraft_java_server: null,
    },
    showStatusBadge: true,
  },
} satisfies Meta<typeof ProjectPageHeader>

export default meta
type Story = StoryObj<typeof meta>

export const Approved: Story = {}
export const Draft: Story = {
  args: {
    projectV3: { ...meta.args.projectV3, review_outcome: 'never_reviewed' },
  },
}
export const InReview: Story = {
  args: { projectV3: { ...meta.args.projectV3, in_review: true } },
}
export const Rejected: Story = {
  args: { projectV3: { ...meta.args.projectV3, review_outcome: 'rejected' } },
}
export const Withheld: Story = {
  args: { projectV3: { ...meta.args.projectV3, review_outcome: 'withheld' } },
}
export const Unlisted: Story = {
  args: { projectV3: { ...meta.args.projectV3, visibility: 'unlisted' } },
}
export const Private: Story = {
  args: { projectV3: { ...meta.args.projectV3, visibility: 'private' } },
}
export const V2Fallback: Story = {
  args: { projectV3: null },
}
