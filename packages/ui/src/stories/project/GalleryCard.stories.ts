import { GripVerticalIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, fn } from 'storybook/test'

import IconButton from '../../components/base/buttons/IconButton.vue'
import GalleryCard from '../../components/project/gallery/GalleryCard.vue'

const stage = new URL('../../../../blog/public/proud-of-you-2026/stage.webp', import.meta.url)

const image = {
	url: stage.href,
	raw_url: stage.href,
	title: 'Pride on the Modrinth SMP',
	description: 'A view of the stage before the parade.',
	created: '2026-06-19T12:00:00Z',
}

const longCaption = {
	...image,
	title: 'The stage and surrounding builds on the Modrinth SMP',
	description:
		'The gathering point for the parade, with seating for the crowd and a stage framed by trees. Community members decorated the paths and built stalls in the surrounding area before the event.',
	created: '2026-10-17T12:00:00Z',
}

type CardProps = InstanceType<typeof GalleryCard>['$props']

const states = [
	{ name: 'Read only', props: { image, heading: 'h2' } },
	{ name: 'Editable', props: { image, editable: true } },
	{ name: 'Actions disabled', props: { image, editable: true, actionsDisabled: true } },
	{ name: 'Banner image', props: { image: { ...image, featured: true } } },
	{ name: 'Title only', props: { image: { ...image, description: undefined } } },
	{ name: 'Description only', props: { image: { ...image, title: undefined } } },
	{ name: 'Image only', props: { image: { ...image, title: undefined, description: undefined } } },
	{ name: 'Missing thumbnail', props: { image: { ...image, url: '', raw_url: undefined } } },
	{ name: 'Long caption', props: { image: longCaption, editable: true } },
	{ name: 'Project settings', props: { image, editable: true }, showSettings: true },
] satisfies { name: string; props: CardProps; showSettings?: boolean }[]

const meta = {
	title: 'Project/Gallery Card',
	component: GalleryCard,
	args: { image, onPreview: fn(), onEdit: fn(), onRemove: fn() },
	render: (args) => ({
		components: { GalleryCard },
		setup: () => ({ args }),
		template: '<div class="max-w-sm"><GalleryCard v-bind="args" /></div>',
	}),
} satisfies Meta<typeof GalleryCard>

export default meta
type Story = StoryObj<typeof meta>

export const AllStates: Story = {
	name: 'All states',
	render: (args) => ({
		components: { GalleryCard, GripVerticalIcon, IconButton },
		setup: () => ({ args, states }),
		template: `
			<div class="grid max-w-6xl grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
				<section v-for="state in states" :key="state.name" class="grid min-w-0 grid-rows-[auto_1fr] gap-2">
					<h2 class="m-0 text-sm font-semibold text-secondary">{{ state.name }}</h2>
					<GalleryCard v-bind="{ ...args, ...state.props }">
						<template v-if="state.showSettings" #thumbnail-overlay>
							<IconButton
								label="Reorder image"
								:circular="false"
								class="!absolute left-2 top-2 z-10 !cursor-grab focus-visible:opacity-100 group-hover/gallery-card:opacity-100 [@media(hover:hover)]:opacity-0"
							>
								<GripVerticalIcon aria-hidden="true" />
							</IconButton>
						</template>
						<template v-if="state.showSettings" #details>
							<p class="m-0 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3 text-sm text-secondary">
								Update the caption to describe what the screenshot shows.
							</p>
						</template>
					</GalleryCard>
				</section>
			</div>
		`,
	}),
}

export const Default: Story = { args: { heading: 'h2' } }
export const Editable: Story = { args: { editable: true } }
export const ActionsDisabled: Story = { args: { editable: true, actionsDisabled: true } }
export const BannerImage: Story = { args: { image: { ...image, featured: true } } }

export const MixedCaptionLengths: Story = {
	render: (args) => ({
		components: { GalleryCard },
		setup: () => ({ args, images: [image, longCaption, { ...image, description: undefined }] }),
		template: `
			<div class="@container max-w-6xl">
				<div class="grid grid-cols-1 gap-4 @[34rem]:grid-cols-2 @[52rem]:grid-cols-3">
					<GalleryCard v-for="(preview, index) in images" :key="index" v-bind="args" :image="preview" :editable="index !== 2" />
				</div>
			</div>
		`,
	}),
	play: async ({ canvasElement }) => {
		const footerBottoms = new Map<number, number>()
		for (const card of canvasElement.querySelectorAll('article')) {
			const heading = card.querySelector('h3')!.getBoundingClientRect()
			const footer = card.querySelector('footer')!
			const date = footer.firstElementChild!.getBoundingClientRect()
			const { bottom } = footer.getBoundingClientRect()
			const { top } = card.getBoundingClientRect()
			await expect(date.left).toBeCloseTo(heading.left)
			await expect(bottom).toBeCloseTo(footerBottoms.get(top) ?? bottom)
			footerBottoms.set(top, bottom)
		}
	},
}

export const NarrowCard: Story = {
	args: { image: longCaption, editable: true },
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="w-52 max-w-full"><story /></div>',
		}),
	],
}
