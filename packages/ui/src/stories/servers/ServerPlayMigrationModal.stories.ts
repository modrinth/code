import type { UploadProgress } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { nextTick, ref } from 'vue'

import { Button } from '../../components/base/buttons'
import { hasCurseForgeUploads } from '../../components/flows/creation-flow-modal/curseforge'
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
				const uploadProgress = ref<UploadProgress | null>(null)

				async function simulateUpload(total: number) {
					for (let loaded = 0; loaded <= total; loaded += total / 20) {
						uploadProgress.value = { loaded, total, progress: loaded / total }
						await new Promise((resolve) => setTimeout(resolve, 150))
					}
					uploadProgress.value = null
					await nextTick()
				}

				const onConfirm = async (source: ServerPlayMigrationSource) => {
					if (source.kind === 'mrpack' && source.file) {
						await simulateUpload(source.file.size || 1)
					} else if (source.kind === 'curseforge' && hasCurseForgeUploads(source.selection)) {
						await simulateUpload(1)
					}
					lastSource.value =
						source.kind === 'mrpack'
							? `mrpack: ${source.file?.name ?? 'existing upload'}`
							: source.kind
					modalRef.value?.hide()
				}
				return { modalRef, lastSource, uploadProgress, onConfirm, kind, options }
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
						:upload-progress="uploadProgress"
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
