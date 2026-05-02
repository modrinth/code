import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { onMounted, onUnmounted, ref } from 'vue'

import BaseTerminal from '../../components/base/BaseTerminal.vue'

const meta = {
	title: 'Base/BaseTerminal',
	component: BaseTerminal,
} satisfies Meta<typeof BaseTerminal>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { BaseTerminal },
		setup() {
			const termRef = ref<InstanceType<typeof BaseTerminal> | null>(null)

			onMounted(() => {
				const t = termRef.value
				if (!t) return
				t.writeln('\x1b[1;32m=== Modrinth Server Console ===\x1b[0m')
				t.writeln('')
				t.writeln('\x1b[36m[10:15:30]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Loading properties')
				t.writeln(
					'\x1b[36m[10:15:30]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Default game type: SURVIVAL',
				)
				t.writeln(
					'\x1b[36m[10:15:31]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Starting Minecraft server on *:25565',
				)
				t.writeln('\x1b[36m[10:15:32]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Preparing level "world"')
				t.writeln(
					"\x1b[36m[10:15:33]\x1b[0m \x1b[33m[Server/WARN]\x1b[0m: Can't keep up! Is the server overloaded?",
				)
				t.writeln(
					'\x1b[36m[10:15:34]\x1b[0m \x1b[31m[Server/ERROR]\x1b[0m: Connection reset by peer',
				)
				t.writeln(
					'\x1b[36m[10:15:35]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Done (4.523s)! For help, type "help"',
				)
			})

			return { termRef }
		},
		template: /*html*/ `
			<div style="width: 100%; height: 95vh;">
				<BaseTerminal ref="termRef" />
			</div>
		`,
	}),
}

const SAMPLE_LINES = [
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Player Steve joined the game',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Steve has made the advancement [Getting an Upgrade]',
	"\x1b[36m[{time}]\x1b[0m \x1b[33m[Server/WARN]\x1b[0m: Can't keep up! Is the server overloaded? Running 2501ms behind",
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Preparing spawn area: 84%',
	'\x1b[36m[{time}]\x1b[0m \x1b[31m[Server/ERROR]\x1b[0m: java.net.ConnectException: Connection timed out',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: [Fabric] Loading 127 mods',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Environment: authlib=6.0.54, java=21.0.3',
	'\x1b[36m[{time}]\x1b[0m \x1b[33m[Server/WARN]\x1b[0m: Ambiguity between arguments at position 1',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Player Alex left the game',
	'\x1b[36m[{time}]\x1b[0m \x1b[31m[Server/ERROR]\x1b[0m: Chunk file at [-3, 12] is missing level data, skipping',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: ThreadedAnvilChunkStorage: All chunks are saved',
	'\x1b[36m[{time}]\x1b[0m \x1b[34m[Server/DEBUG]\x1b[0m: Reloading ResourceManager: Default, fabric',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: <Steve> Hello everyone!',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Saving the game (this may take a moment!)',
	'\x1b[36m[{time}]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Saved the game',
]

function getTimeString(): string {
	const now = new Date()
	return `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}:${String(now.getSeconds()).padStart(2, '0')}`
}

export const StreamingLogs: StoryObj = {
	render: () => ({
		components: { BaseTerminal },
		setup() {
			const termRef = ref<InstanceType<typeof BaseTerminal> | null>(null)
			let interval: ReturnType<typeof setInterval> | null = null
			let index = 0

			onMounted(() => {
				termRef.value?.writeln('\x1b[1;32m=== Modrinth Server Console ===\x1b[0m')
				termRef.value?.writeln('')

				interval = setInterval(() => {
					const line = SAMPLE_LINES[index % SAMPLE_LINES.length].replace('{time}', getTimeString())
					termRef.value?.writeln(line)
					index++
				}, 25)
			})

			onUnmounted(() => {
				if (interval) clearInterval(interval)
			})

			return { termRef }
		},
		template: /*html*/ `
			<div style="width: 100%; height: 95vh;">
				<BaseTerminal ref="termRef" />
			</div>
		`,
	}),
}

export const WithInput: StoryObj = {
	render: () => ({
		components: { BaseTerminal },
		setup() {
			const termRef = ref<InstanceType<typeof BaseTerminal> | null>(null)

			const onCommand = (cmd: string) => {
				termRef.value?.writeln(`\x1b[32m> ${cmd}\x1b[0m`)
			}

			onMounted(() => {
				termRef.value?.writeln('\x1b[1;32m=== Modrinth Server Console ===\x1b[0m')
				termRef.value?.writeln('')
				termRef.value?.writeln(
					'\x1b[36m[10:15:35]\x1b[0m \x1b[32m[Server/INFO]\x1b[0m: Done! For help, type "help"',
				)
			})

			return { termRef, onCommand }
		},
		template: /*html*/ `
			<div style="width: 100%; height: 95vh;">
				<BaseTerminal ref="termRef" show-input @command="onCommand" />
			</div>
		`,
	}),
}
