import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectStatusLink from '../../components/project/ProjectStatusLink.vue'

const meta = {
  title: 'Project/ProjectStatusLink',
  component: ProjectStatusLink,
  args: {
    project: {
      name: 'Example project',
      slug: 'example-project',
      visibility: 'listed',
      in_review: false,
      review_outcome: 'approved',
    },
  },
} satisfies Meta<typeof ProjectStatusLink>

export default meta
type Story = StoryObj<typeof meta>

export const Approved: Story = {}
export const Draft: Story = {
  args: { project: { ...meta.args.project, review_outcome: 'never_reviewed' } },
}
export const InReview: Story = {
  args: { project: { ...meta.args.project, in_review: true } },
}
export const Rejected: Story = {
  args: { project: { ...meta.args.project, review_outcome: 'rejected' } },
}
export const Withheld: Story = {
  args: { project: { ...meta.args.project, review_outcome: 'withheld' } },
}
export const Unlisted: Story = {
  args: { project: { ...meta.args.project, visibility: 'unlisted' } },
}
export const Private: Story = {
  args: { project: { ...meta.args.project, visibility: 'private' } },
}
