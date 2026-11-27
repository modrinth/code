import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { defineComponent, h, onBeforeUnmount, onMounted, ref } from 'vue'

import FileTableHeader from '../../layouts/shared/files-tab/components/FileTableHeader.vue'
import FileTableRow from '../../layouts/shared/files-tab/components/FileTableRow.vue'
import {
	type FileManagerContext,
	provideFileManager,
} from '../../layouts/shared/files-tab/providers/file-manager'
import type { FileItem } from '../../layouts/shared/files-tab/types'

const now = Math.floor(Date.now() / 1000)

const items: FileItem[] = [
	{ name: 'config', type: 'directory', path: 'config', count: 42, created: now, modified: now },
	{ name: 'mods', type: 'directory', path: 'mods', count: 118, created: now, modified: now },
	{ name: 'world', type: 'directory', path: 'world', count: 18, created: now, modified: now },
	{
		name: 'banned-ips.json',
		type: 'file',
		path: 'banned-ips.json',
		size: 2,
		created: now,
		modified: now,
	},
	{ name: 'eula.txt', type: 'file', path: 'eula.txt', size: 10, created: now, modified: now },
	{ name: 'ops.json', type: 'file', path: 'ops.json', size: 1208, created: now, modified: now },
	{ name: 'run.sh', type: 'file', path: 'run.sh', size: 366, created: now, modified: now },
]

const noop = async () => {}

const FileManagerStub = defineComponent({
	setup(_, { slots }) {
		provideFileManager({
			items: ref(items),
			loading: ref(false),
			error: ref(null),
			currentPath: ref('/'),
			navigateTo: () => {},
			editingFile: ref(null),
			startEditing: () => {},
			stopEditing: () => {},
			createItem: noop,
			renameItem: noop,
			moveItem: noop,
			deleteItem: noop,
			readFile: async () => '',
			readFileAsBlob: async () => new Blob(),
			writeFile: noop,
			downloadFile: noop,
			uploadFiles: () => {},
			refresh: () => {},
		} satisfies FileManagerContext)
		return () => slots.default?.()
	},
})

const RootFontSize = defineComponent({
	props: { size: { type: String, required: true } },
	setup(props, { slots }) {
		onMounted(() => {
			document.documentElement.style.fontSize = props.size
		})
		onBeforeUnmount(() => {
			document.documentElement.style.fontSize = ''
		})
		return () => h('div', slots.default?.())
	},
})

const tableTemplate = /* html */ `
	<div class="@container relative flex flex-col overflow-clip rounded-[20px] border border-solid border-surface-4 shadow-sm">
		<FileTableHeader
			sort-field="name"
			:sort-desc="false"
			:all-selected="false"
			:some-selected="false"
			:is-stuck="false"
		/>
		<FileTableRow
			v-for="(item, idx) in items"
			:key="item.path"
			v-bind="item"
			:index="idx"
			:is-last="idx === items.length - 1"
			:selected="false"
		/>
	</div>
`

const meta = {
	title: 'Files/FileTable',
	component: FileTableRow,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story, FileManagerStub },
			template: '<FileManagerStub><story /></FileManagerStub>',
		}),
	],
} satisfies Meta<typeof FileTableRow>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { FileTableHeader, FileTableRow },
		setup() {
			return { items }
		},
		template: tableTemplate,
	}),
}

export const LargeBrowserFont: StoryObj = {
	render: () => ({
		components: { FileTableHeader, FileTableRow, RootFontSize },
		setup() {
			return { items }
		},
		template: `<RootFontSize size="22px">${tableTemplate}</RootFontSize>`,
	}),
}
