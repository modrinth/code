import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import type { UpdateAllItem, UpdateAllSelection } from '../../components/modal'
import UpdateAllModal from '../../components/modal/update-all-modal/index.vue'

const changelog = `### Additions
- Added trades for Saccharine Saplings, Hearty Grains, Chipped Pot, and Masterpiece Teacup to the Wandering Trader.
- Added brewing recipe for Throat Spray.
- Added vanilla sprinting's field of view change to Pokémon land mounts.
- Added a customizable keybind for the riding freeview look button.
- Added double tap to sprint on land mounts.

### Model updates for the following Pokémon
- Tinkatink
- Tinkatuff
- Tinkaton

### Changes
- Made Jet pitch turning 1.5x faster across all Pokémon.
- Party Pokémon now attack only mobs that are attacking their owner, instead of randomly targeting unrelated mobs and getting themselves hurt.
- Reduced the cost of the Vivichoke Seed trade with the Wandering Trader.

### Fixes
- Fixed incorrect camera pivot on Bird, Jet, and Dolphin mounts, leading to some disorienting riding.
- Fixed players suffocating while on vanilla mounts.
- Fixed a graphics crash that could occur on some machines when campfire pots were nearby.
- Fixed ride controls overlay being displayed to passengers.
- Fixed ridden Pokémon land collision and some issues around stepping up blocks.
- Fixed friendship being reset to the default value when evolving a Pokémon.
- Fixed Combees not depositing honey upon leaving a hive if they entered it with nectar.
- Fixed a crash related to NPC navigation.
`

const items: UpdateAllItem[] = [
	{
		id: 'mods/iris.jar',
		project: {
			id: 'YL57xq9U',
			title: 'Iris Shaders',
			icon_url:
				'https://cdn.modrinth.com/data/YL57xq9U/18d0e7f076d3d6ed5bedd472b853909aac5da202_96.webp',
		},
		currentVersion: { id: 'iris-current', version_number: '1.11.2+26.1-neoforge' },
		versions: [
			{
				id: 'iris-next',
				version_number: '1.11.3+26.1-neoforge',
				version_type: 'release',
				changelog,
			},
		],
	},
	{
		id: 'shaderpacks/complementary.zip',
		project: {
			id: 'HVnmMxH1',
			title: 'Complementary Shaders',
			icon_url:
				'https://cdn.modrinth.com/data/HVnmMxH1/79cb7c8123bbc54945305b2ebad6b8881efdf5f8_96.webp',
		},
		currentVersion: { id: 'complementary-current', version_number: 'r5.8' },
		initiallySelected: false,
		versions: [
			{
				id: 'complementary-alpha',
				version_number: 'r5.9',
				version_type: 'alpha',
				changelog: '### Experimental changes\n- Updated lighting and water reflections.',
			},
			{
				id: 'complementary-release',
				version_number: 'r5.8.1',
				version_type: 'release',
				changelog: '### Fixes\n- Fixed water reflections.',
			},
		],
	},
	{
		id: 'mods/cobblemon.jar',
		project: {
			id: 'MdwFAVRL',
			title: 'Cobblemon',
			icon_url:
				'https://cdn.modrinth.com/data/MdwFAVRL/abfca1654a2d09bab85cbffcc9869938c951ee0e_96.webp',
		},
		currentVersion: { id: 'cobblemon-current', version_number: '1.7.2' },
		versions: [
			{ id: 'cobblemon-next', version_number: '1.8.0', version_type: 'release', changelog },
			{
				id: 'cobblemon-patch',
				version_number: '1.7.3',
				version_type: 'release',
				changelog: '### Fixes\n- Fixed mount collisions.',
			},
		],
	},
	{
		id: 'mods/create.jar',
		project: {
			id: 'LNytGWDc',
			title: 'Create',
			icon_url:
				'https://cdn.modrinth.com/data/LNytGWDc/61d716699bcf1ec42ed4926a9e1c7311be6087e2_96.webp',
		},
		currentVersion: { id: 'create-current', version_number: 'mc1.21.1-6.0.9' },
		versions: [
			{
				id: 'create-next',
				version_number: '6.0.10+mc1.21.1',
				version_type: 'release',
				changelog: '### Fixes\n- Improved contraption stability.',
			},
		],
	},
	{
		id: 'mods/aeronautics.jar',
		project: {
			id: 'oWaK0Q19',
			title: 'Create Aeronautics',
			icon_url:
				'https://cdn.modrinth.com/data/oWaK0Q19/f66b5589924884ffd81acb27f3ccb775867a962e_96.webp',
		},
		currentVersion: { id: 'aeronautics-current', version_number: '1.3.0+mc1.21.1' },
		versions: [
			{
				id: 'aeronautics-next',
				version_number: '1.3.2+mc1.21.1',
				version_type: 'release',
				changelog: '### Changes\n- Improved flight controls.',
			},
			{
				id: 'aeronautics-patch',
				version_number: '1.3.1+mc1.21.1',
				version_type: 'release',
				changelog: '### Fixes\n- Fixed aircraft collisions.',
			},
		],
	},
	{
		id: 'mods/sable.jar',
		project: {
			id: 'T9PomCSv',
			title: 'Sable',
			icon_url:
				'https://cdn.modrinth.com/data/T9PomCSv/8c0a8c64c9a5a8d446d0aa23d244cb9b52314a1d.png',
		},
		currentVersion: { id: 'sable-current', version_number: '2.0.4+mc1.21.1' },
		versions: [
			{
				id: 'sable-next',
				version_number: '2.0.5+mc1.21.1',
				version_type: 'release',
				changelog: '### Fixes\n- Fixed compatibility with the latest Create release.',
			},
		],
	},
	{
		id: 'mods/sodium.jar',
		project: {
			id: 'AANobbMI',
			title: 'Sodium',
			icon_url:
				'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
		},
		currentVersion: { id: 'sodium-current', version_number: 'mc1.21.1-0.8.13-neoforge' },
		versions: [
			{
				id: 'sodium-next',
				version_number: 'mc26.2-0.9.2-beta.1-neoforge',
				version_type: 'beta',
				changelog:
					'### Performance\n- Improved chunk rendering.\n\n### Fixes\n- Fixed rendering artifacts.',
			},
		],
	},
]

const meta = {
	title: 'Instances/UpdateAllModal',
	component: UpdateAllModal,
	parameters: { layout: 'centered' },
	args: {
		items,
		onUpdate: fn(),
		onCancel: fn(),
		onChangelog: fn(),
	},
	argTypes: {
		items: { description: 'Content items and compatible update candidates, ordered by preference.' },
		loading: { control: 'boolean' },
		loadingChangelog: { control: 'boolean' },
		actionLoading: { control: 'boolean' },
		actionDisabled: { control: 'boolean' },
	},
} satisfies Meta<typeof UpdateAllModal>

export default meta
type Story = StoryObj<typeof meta>

function renderModal(changelogItemId?: string): NonNullable<Story['render']> {
	return (args) => ({
		components: { UpdateAllModal, Button },
		setup() {
			const modal = ref<InstanceType<typeof UpdateAllModal>>()
			const selected = ref<UpdateAllSelection[]>([])
			function open() {
				selected.value = []
				modal.value?.show({ changelogItemId })
			}
			function update(selections: UpdateAllSelection[]) {
				selected.value = selections
				args.onUpdate?.(selections)
			}
			return { args, modal, selected, open, update }
		},
		template: `
			<div class="flex max-w-xl flex-col items-start gap-4">
				<Button type="colored" color="brand" @click="open">Update all</Button>
				<UpdateAllModal ref="modal" v-bind="args" :on-update="update" />
				<div v-if="selected.length" role="status" class="text-primary">
					<p class="font-semibold text-contrast">Requested updates</p>
					<ul>
						<li v-for="selection in selected" :key="selection.id">
							{{ selection.id }} → {{ selection.version.version_number }}
						</li>
					</ul>
				</div>
			</div>
		`,
	})
}

export const Default: Story = {
	render: renderModal(),
	parameters: {
		docs: {
			description: {
				story: 'Open and close a project changelog to see the panel slide over the unchanged table.',
			},
		},
	},
}

export const ChangelogOpen: Story = {
	render: renderModal('mods/iris.jar'),
	parameters: {
		docs: {
			description: {
				story: 'Click a project row to view its changelog. The checkbox still selects the project, and modifier-clicks still change selection.',
			},
		},
	},
}

export const ChangelogVersionSelection: Story = { render: renderModal('mods/cobblemon.jar') }

export const AllSelected: Story = {
	render: renderModal(),
	args: { items: items.map((item) => ({ ...item, initiallySelected: true })) },
	parameters: {
		docs: {
			description: {
				story: 'Shift-click a selected row, then another selected row, to deselect the range between them. A third Shift-click starts a new range.',
			},
		},
	},
}

export const NoneSelected: Story = {
	render: renderModal(),
	args: { items: items.map((item) => ({ ...item, initiallySelected: false })) },
	parameters: {
		docs: {
			description: {
				story: 'Shift-click two project rows to select the range between them. The next Shift-click starts a new range. Ctrl-click or Command-click a row to toggle it individually.',
			},
		},
	},
}

export const SingleProject: Story = {
	render: renderModal(),
	args: { items: [items[0]] },
}

export const Loading: Story = {
	render: renderModal(),
	args: { items: [], loading: true },
}

export const Empty: Story = { render: renderModal(), args: { items: [] } }

export const LoadingChangelog: Story = {
	render: renderModal('mods/iris.jar'),
	args: { loadingChangelog: true },
}

export const NoChangelog: Story = {
	render: renderModal('mods/iris.jar'),
	args: {
		items: items.map((item) => ({
			...item,
			versions: item.versions.map((version) => ({ ...version, changelog: '' })),
		})),
	},
}

export const NoCompatibleUpdates: Story = {
	render: renderModal(),
	args: { items: items.map((item) => ({ ...item, versions: [] })) },
}

export const UpdateDisabled: Story = {
	render: renderModal(),
	args: { actionDisabled: true },
}

export const Updating: Story = {
	render: renderModal(),
	args: { actionLoading: true },
}

export const ManyProjects: Story = {
	render: renderModal(),
	args: {
		items: Array.from({ length: 50 }, (_, index) => ({
			...items[index % items.length],
			id: `mods/example-${index}.jar`,
			project: {
				...items[index % items.length].project,
				title: `${items[index % items.length].project.title} ${index + 1}`,
			},
		})),
	},
}

export const LongNames: Story = {
	render: renderModal(),
	args: {
		items: items.map((item) => ({
			...item,
			project: { ...item.project, title: `${item.project.title} with a very long project name` },
			versions: item.versions.map((version) => ({
				...version,
				version_number: `${version.version_number}-experimental-build-with-a-long-version-number`,
			})),
		})),
	},
}
