import { ref } from 'vue'

import type { JavaVersion } from '@/helpers/instance'
import { get_jre } from '@/helpers/jre.js'

export type JavaStatus = 'empty' | 'checking' | 'valid' | 'invalid'

export default function useJavaTest() {
	const status = ref<JavaStatus>('empty')
	let latestCheck = 0

	async function checkJava(path: string | undefined, majorVersion: number | null) {
		const check = ++latestCheck
		if (!path) {
			status.value = 'empty'
			return
		}

		status.value = 'checking'
		const result: JavaVersion | null = await get_jre(path).catch(() => null)
		if (check !== latestCheck) return

		status.value =
			result && (majorVersion === null || result.parsed_version === majorVersion)
				? 'valid'
				: 'invalid'
	}

	return { status, checkJava }
}
