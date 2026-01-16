import type { Meta, StoryObj } from '@storybook/vue3-vite'

import AutoBrandIcon from '../../components/base/AutoBrandIcon.vue'

const meta = {
	title: 'Base/AutoBrandIcon',
	component: AutoBrandIcon,
} satisfies Meta<typeof AutoBrandIcon>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		keyword: 'discord',
	},
}

export const AllBrands: StoryObj = {
	render: () => ({
		components: { AutoBrandIcon },
		template: `
			<div class="grid grid-cols-4 gap-4">
				<div v-for="brand in brands" :key="brand" class="flex flex-col items-center gap-2 p-4 bg-bg-raised rounded-lg">
					<AutoBrandIcon :keyword="brand" class="h-8 w-8" />
					<span class="text-sm text-secondary">{{ brand }}</span>
				</div>
			</div>
		`,
		setup() {
			const brands = [
				'discord',
				'github',
				'twitter',
				'youtube',
				'twitch',
				'reddit',
				'mastodon',
				'bluesky',
				'patreon',
				'kofi',
				'paypal',
				'curseforge',
				'modrinth',
				'instagram',
				'facebook',
				'tiktok',
			]
			return { brands }
		},
	}),
}
