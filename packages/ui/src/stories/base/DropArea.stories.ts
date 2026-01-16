import type { Meta, StoryObj } from '@storybook/vue3-vite'

import DropArea from '../../components/base/DropArea.vue'

const meta = {
	title: 'Base/DropArea',
	component: DropArea,
} satisfies Meta<typeof DropArea>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: /*html*/ `
			<DropArea accept="*" @change="(files) => console.log('Files dropped:', files)">
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drag and drop files anywhere on the page</p>
					<p class="text-sm text-secondary mt-2">The drop overlay will appear when you drag files over</p>
				</div>
			</DropArea>
		`,
	}),
}

export const ImagesOnly: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: /*html*/ `
			<DropArea accept="image/*" @change="(files) => console.log('Images dropped:', files)">
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drop images here</p>
					<p class="text-sm text-secondary mt-2">Only accepts image files</p>
				</div>
			</DropArea>
		`,
	}),
}

export const AcceptMods: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: /*html*/ `
			<DropArea
				accept=".jar,.zip,.litemod,application/java-archive,application/x-java-archive,application/zip,.sig,.asc,.gpg,application/pgp-signature,application/pgp-keys"
				@change="(files) => console.log('Mod files dropped:', files)"
			>
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drop mod files here</p>
					<p class="text-sm text-secondary mt-2">Accepts .jar, .zip, .litemod, and signature files (.sig, .asc, .gpg)</p>
				</div>
			</DropArea>
		`,
	}),
}

export const AcceptImages: StoryObj = {
	render: () => ({
		components: { DropArea },
		template: /*html*/ `
			<DropArea
				accept="image/png,image/jpeg,image/gif,image/webp,image/svg+xml"
				@change="(files) => console.log('Image files dropped:', files)"
			>
				<div class="p-8 border-2 border-dashed border-divider rounded-lg text-center">
					<p class="text-secondary">Drop image files here</p>
					<p class="text-sm text-secondary mt-2">Accepts PNG, JPEG, GIF, WebP, and SVG images</p>
				</div>
			</DropArea>
		`,
	}),
}
