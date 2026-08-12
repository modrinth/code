import type { Archon, UploadState } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { computed, ref } from 'vue'

import InstallingBanner from '../../components/servers/InstallingBanner.vue'
import type { ServerInstallationState } from '../../composables/server-installation-tracker'
import { provideModrinthServerContext } from '../../providers'
import type {
	CancelUploadHandler,
	ModrinthServerContext,
	ServerStats,
} from '../../providers/server-context'

function renderInstallation(state: ServerInstallationState) {
	return () => ({
		components: { InstallingBanner },
		setup() {
			const installation = ref<ServerInstallationState | null>(state)
			const serverContext: ModrinthServerContext = {
				get serverId() {
					return 'story-server'
				},
				worldId: ref<string | null>('story-world'),
				server: ref({
					server_id: 'story-server',
					status: 'installing',
				} as Archon.Servers.v0.Server),
				serverFull: computed(() => null),
				currentUserPermissions: computed(() => 0),
				isConnected: ref(true),
				isWsAuthIncorrect: ref(false),
				powerState: ref('stopped'),
				powerStateDetails: ref(undefined),
				isServerRunning: computed(() => false),
				stats: ref<ServerStats>({
					current: {
						cpu_percent: 0,
						ram_usage_bytes: 0,
						ram_total_bytes: 1,
						storage_usage_bytes: 0,
						storage_total_bytes: 0,
					},
					past: {
						cpu_percent: 0,
						ram_usage_bytes: 0,
						ram_total_bytes: 1,
						storage_usage_bytes: 0,
						storage_total_bytes: 0,
					},
					graph: { cpu: [], ram: [] },
				}),
				uptimeSeconds: ref(0),
				installProgressItems: ref<Archon.Websocket.v0.InstallProgressItem[]>([]),
				installation: computed(() => installation.value),
				beginInstallation: () => {},
				cancelOptimisticInstallation: () => {},
				dismissInstallation: () => {
					installation.value = null
				},
				busyReasons: computed(() => []),
				fsAuth: ref(null),
				fsOps: ref<Archon.Websocket.v0.FilesystemOperation[]>([]),
				fsQueuedOps: ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([]),
				refreshFsAuth: async () => {},
				uploadState: ref<UploadState>({
					isUploading: false,
					currentFileName: null,
					currentFileProgress: 0,
					uploadedBytes: 0,
					totalBytes: 0,
					completedFiles: 0,
					totalFiles: 0,
				}),
				cancelUpload: ref<CancelUploadHandler | null>(null),
				activeOperations: computed(() => []),
				dismissOperation: async () => {},
			}
			provideModrinthServerContext(serverContext)
		},
		template: '<InstallingBanner />',
	})
}

const meta = {
	title: 'Servers/InstallingBanner',
	component: InstallingBanner,
} satisfies Meta<typeof InstallingBanner>

export default meta
type Story = StoryObj<typeof meta>

export const PlatformPending: Story = {
	render: renderInstallation({
		id: 'platform:fabric:0.16.14:1.21.1',
		key: {
			type: 'platform',
			platform: 'fabric',
			platform_version: '0.16.14',
			game_version: '1.21.1',
		},
		status: 'pending',
		progress: null,
		error: null,
		source: 'optimistic',
	}),
}

export const PlatformProgress: Story = {
	render: renderInstallation({
		id: 'platform:fabric:0.16.14:1.21.1',
		key: {
			type: 'platform',
			platform: 'fabric',
			platform_version: '0.16.14',
			game_version: '1.21.1',
		},
		status: 'installing',
		progress: 45,
		error: null,
		source: 'websocket',
	}),
}

export const Vanilla: Story = {
	render: renderInstallation({
		id: 'platform:vanilla::1.21.1',
		key: {
			type: 'platform',
			platform: 'vanilla',
			platform_version: '',
			game_version: '1.21.1',
		},
		status: 'installing',
		progress: 20,
		error: null,
		source: 'websocket',
	}),
}

export const Modpack: Story = {
	render: renderInstallation({
		id: 'modrinth-modpack:project:version',
		key: {
			type: 'modrinth_modpack',
			project_id: 'project',
			version_id: 'version',
		},
		status: 'installing',
		progress: 72,
		error: null,
		source: 'websocket',
	}),
}

export const PlatformFailed: Story = {
	render: renderInstallation({
		id: 'platform:quilt:0.30.1-beta.2:1.14.4',
		key: {
			type: 'platform',
			platform: 'quilt',
			platform_version: '0.30.1-beta.2',
			game_version: '1.14.4',
		},
		status: 'failed',
		progress: null,
		error: 'Platform installer failed',
		source: 'websocket',
	}),
}

export const ModpackFailed: Story = {
	render: renderInstallation({
		id: 'modrinth-modpack:project:version',
		key: {
			type: 'modrinth_modpack',
			project_id: 'project',
			version_id: 'version',
		},
		status: 'failed',
		progress: null,
		error: 'The modpack does not include a downloadable file',
		source: 'websocket',
	}),
}

export const LocalModpackFailed: Story = {
	render: renderInstallation({
		id: 'local-modpack:example.mrpack',
		key: {
			type: 'local_modpack',
			filename: 'example.mrpack',
		},
		status: 'failed',
		progress: null,
		error: 'The modpack could not be read',
		source: 'websocket',
	}),
}
