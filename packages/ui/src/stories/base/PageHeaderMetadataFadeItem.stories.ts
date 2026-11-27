import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import PageHeaderMetadataFadeItem from '../../components/base/page-header/metadata/fade-item.vue'
import PageHeaderMetadata from '../../components/base/page-header/metadata/index.vue'
import PageHeaderMetadataItem from '../../components/base/page-header/metadata/page-header-metadata-item.vue'

const meta = {
	title: 'Base/PageHeaderMetadataFadeItem',
	component: PageHeaderMetadataFadeItem,
} satisfies Meta<typeof PageHeaderMetadataFadeItem>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { PageHeaderMetadata, PageHeaderMetadataItem, PageHeaderMetadataFadeItem },
		setup() {
			const shown = ref(true)
			return { shown }
		},
		template: /* html */ `
			<div class="flex flex-col gap-4">
				<label class="flex items-center gap-2">
					<input v-model="shown" type="checkbox" />
					Shown
				</label>
				<PageHeaderMetadata>
					<PageHeaderMetadataItem>Fabric 1.21.1</PageHeaderMetadataItem>
					<PageHeaderMetadataFadeItem :shown="shown">Installing content</PageHeaderMetadataFadeItem>
				</PageHeaderMetadata>
			</div>
		`,
	}),
}
