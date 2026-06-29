import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import FileTreeSelect, { type FileTreeSelectItem } from '../../components/base/FileTreeSelect.vue'

const modified = Math.floor(new Date('2026-06-29T10:30:00Z').getTime() / 1000)

const MODPACK_FILES: FileTreeSelectItem[] = [
	{ path: 'config/fabric_loader_dependencies.json', size: 918, modified: modified - 600 },
	{ path: 'config/modmenu.json', size: 2401, modified: modified - 7200 },
	{ path: 'config/iris.properties', size: 1208, modified: modified - 5400 },
	{ path: 'config/crash_assistant/settings.toml', size: 714, modified: modified - 1200 },
	{ path: 'config/defaultoptions/options.txt', size: 4820, modified: modified - 3200 },
	{ path: 'config/defaultoptions/keybindings.txt', size: 3012, modified: modified - 3300 },
	{ path: 'mods/sodium-fabric-0.6.13+mc1.21.6.jar', size: 1290240, modified: modified - 900 },
	{ path: 'mods/iris-fabric-1.8.8+mc1.21.6.jar', size: 2782400, modified: modified - 1100 },
	{ path: 'mods/modmenu-15.0.0-beta.3.jar', size: 824220, modified: modified - 1800 },
	{ path: 'resourcepacks/FreshAnimations_v1.9.3.zip', size: 4382210, modified: modified - 2600 },
	{ path: 'resourcepacks/Mod Menu Helper.zip', size: 104522, modified: modified - 2800 },
	{ path: 'shaderpacks/ComplementaryUnbound_r5.5.1.zip', size: 23882210, modified: modified - 3400 },
	{ path: 'datapacks/terralith.zip', size: 17452213, modified: modified - 4200 },
	{ path: 'icon.png', size: 128044, modified: modified - 5000 },
	{ path: 'profile.json', size: 928, modified: modified - 400, disabled: true },
	{ path: 'modrinth_logs/launcher.log', size: 224018, modified: modified - 100, disabled: true },
]

const meta = {
	title: 'Base/FileTreeSelect',
	component: FileTreeSelect,
} satisfies Meta<typeof FileTreeSelect>

export default meta

export const ModpackExport: StoryObj = {
	render: () => ({
		components: { FileTreeSelect },
		setup() {
			const selected = ref([
				'config/fabric_loader_dependencies.json',
				'config/crash_assistant/settings.toml',
				'config/defaultoptions/options.txt',
				'mods/sodium-fabric-0.6.13+mc1.21.6.jar',
				'mods/iris-fabric-1.8.8+mc1.21.6.jar',
				'resourcepacks/FreshAnimations_v1.9.3.zip',
				'shaderpacks/ComplementaryUnbound_r5.5.1.zip',
			])
			const selectedLabel = computed(() => `${selected.value.length} selected`)

			return {
				items: MODPACK_FILES,
				selected,
				selectedLabel,
			}
		},
		template: /*html*/ `
			<div class="max-w-2xl">
				<FileTreeSelect v-model="selected" :items="items" />
				<div class="mt-4 rounded-[20px] bg-surface-3 p-4 text-sm text-secondary">
					<div class="font-semibold text-contrast">{{ selectedLabel }}</div>
					<div class="mt-2 flex flex-col gap-1">
						<span v-for="path in selected" :key="path" class="truncate">{{ path }}</span>
					</div>
				</div>
			</div>
		`,
	}),
}

export const EmptyRoot: StoryObj = {
	render: () => ({
		components: { FileTreeSelect },
		setup() {
			const selected = ref<string[]>([])
			return { selected }
		},
		template: /*html*/ `
			<div class="max-w-2xl">
				<FileTreeSelect v-model="selected" :items="[]" />
			</div>
		`,
	}),
}
