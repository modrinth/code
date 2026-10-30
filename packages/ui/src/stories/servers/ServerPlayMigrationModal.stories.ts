import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import ServerPlayMigrationModal from '../../components/servers/play-migration/ServerPlayMigrationModal.vue'
import type {
	ServerPlayMigrationKind,
	ServerPlayMigrationSource,
} from '../../components/servers/play-migration/types'

const meta = {
	title: 'Servers/ServerPlayMigrationModal',
	component: ServerPlayMigrationModal,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ServerPlayMigrationModal>

export default meta
type Story = StoryObj<typeof meta>

function migrationStory(
	kind: ServerPlayMigrationKind,
	options: { modpackName?: string; hasUploadedMrpack?: boolean } = {},
): Story {
	return {
		args: { kind, worldId: null },
		render: () => ({
			components: { ServerPlayMigrationModal, Button },
			setup() {
				const modalRef = ref<InstanceType<typeof ServerPlayMigrationModal> | null>(null)
				const lastSource = ref('')
				const onConfirm = (source: ServerPlayMigrationSource) => {
					lastSource.value =
						source.kind === 'mrpack'
							? `mrpack: ${source.file?.name ?? 'existing upload'}`
							: source.kind
				}
				return { modalRef, lastSource, onConfirm, kind, options }
			},
			template: /*html*/ `
				<div class="flex flex-col items-center gap-4">
					<Button type="colored" color="brand" @click="modalRef?.show()">Get server ready</Button>
					<p v-if="lastSource" class="text-sm text-secondary">Confirmed with: {{ lastSource }}</p>
					<ServerPlayMigrationModal
						ref="modalRef"
						:kind="kind"
						:modpack-name="options.modpackName"
						:has-uploaded-mrpack="options.hasUploadedMrpack"
						world-id="00000000-0000-0000-0000-000000000000"
						@confirm="onConfirm"
					/>
				</div>
			`,
		}),
	}
}

export const ModrinthModpack = migrationStory('modrinth', { modpackName: 'Cobblemon 1.8.4' })

export const CustomSetup = migrationStory('custom')

export const ExternalMrpack = migrationStory('mrpack')

export const ExternalMrpackStillUploaded = migrationStory('mrpack', { hasUploadedMrpack: true })

export const CurseForgeModpack = migrationStory('curseforge')
