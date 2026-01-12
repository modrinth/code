import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Avatar from '../../components/base/Avatar.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ContentPageHeader from '../../components/base/ContentPageHeader.vue'

const meta = {
	title: 'Base/ContentPageHeader',
	component: ContentPageHeader,
} satisfies Meta<typeof ContentPageHeader>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { ContentPageHeader, Avatar, ButtonStyled },
		template: `
			<ContentPageHeader>
				<template #icon>
					<Avatar size="64px" />
				</template>
				<template #title>Project Name</template>
				<template #summary>A brief description of the project goes here.</template>
				<template #stats>
					<span>1.2M downloads</span>
					<span>50K followers</span>
				</template>
				<template #actions>
					<ButtonStyled color="brand">
						<button>Follow</button>
					</ButtonStyled>
				</template>
			</ContentPageHeader>
		`,
	}),
}

export const WithTitleSuffix: Story = {
	render: () => ({
		components: { ContentPageHeader, Avatar, ButtonStyled },
		template: `
			<ContentPageHeader>
				<template #icon>
					<Avatar size="64px" />
				</template>
				<template #title>Featured Project</template>
				<template #title-suffix>
					<span class="px-2 py-1 bg-brand-highlight text-brand rounded-full text-sm">Featured</span>
				</template>
				<template #summary>This project has been featured by the Modrinth team.</template>
				<template #actions>
					<ButtonStyled color="brand">
						<button>Download</button>
					</ButtonStyled>
					<ButtonStyled type="transparent">
						<button>Share</button>
					</ButtonStyled>
				</template>
			</ContentPageHeader>
		`,
	}),
}
