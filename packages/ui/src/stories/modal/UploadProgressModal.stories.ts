import type { UploadHandle, UploadProgress } from '@modrinth/api-client'
import type { StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import UploadProgressModal from '../../components/modal/UploadProgressModal.vue'

const meta = {
	title: 'Modal/UploadProgressModal',
	component: UploadProgressModal,
}

export default meta
type Story = StoryObj<typeof UploadProgressModal>

function createMockUploadHandle(totalBytes: number, durationMs: number): UploadHandle<void> {
	let progressCallback: ((progress: UploadProgress) => void) | null = null
	let cancelled = false

	const promise = new Promise<void>((resolve, reject) => {
		const startTime = Date.now()
		const interval = setInterval(() => {
			if (cancelled) {
				clearInterval(interval)
				reject(new Error('Upload cancelled'))
				return
			}
			const elapsed = Date.now() - startTime
			const progress = Math.min(elapsed / durationMs, 1)
			const loaded = Math.round(totalBytes * progress)
			progressCallback?.({ loaded, total: totalBytes, progress })
			if (progress >= 1) {
				clearInterval(interval)
				resolve()
			}
		}, 50)
	})

	const handle: UploadHandle<void> = {
		promise,
		onProgress(callback) {
			progressCallback = callback
			return handle
		},
		cancel() {
			cancelled = true
		},
	}

	return handle
}

export const Default: Story = {
	render: () => ({
		components: { UploadProgressModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof UploadProgressModal> | null>(null)
			const startUpload = () => {
				const handle = createMockUploadHandle(50 * 1024 * 1024, 3000)
				modalRef.value?.track(handle)
			}
			return { modalRef, startUpload }
		},
		template: `
			<div>
				<ButtonStyled color="brand">
					<button @click="startUpload">Upload 50MB file (3s)</button>
				</ButtonStyled>
				<UploadProgressModal ref="modalRef" />
			</div>
		`,
	}),
}

export const LargeFile: Story = {
	render: () => ({
		components: { UploadProgressModal, ButtonStyled },
		setup() {
			const modalRef = ref<InstanceType<typeof UploadProgressModal> | null>(null)
			const startUpload = () => {
				const handle = createMockUploadHandle(500 * 1024 * 1024, 8000)
				modalRef.value?.track(handle)
			}
			return { modalRef, startUpload }
		},
		template: `
			<div>
				<ButtonStyled color="brand">
					<button @click="startUpload">Upload 500MB file (8s)</button>
				</ButtonStyled>
				<UploadProgressModal ref="modalRef" />
			</div>
		`,
	}),
}
