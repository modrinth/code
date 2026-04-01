import { ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { test_jre } from '@/helpers/jre.js'

export default function useJavaTest() {
	const testingJava = ref(false)
	const javaTestResult = ref<boolean | null>(null)
	let testDebounceTimer: ReturnType<typeof setTimeout> | null = null

	async function runJavaTest(path: string, version: number, track = true) {
		if (testDebounceTimer) {
			clearTimeout(testDebounceTimer)
			testDebounceTimer = null
		}
		if (!path) {
			javaTestResult.value = null
			return
		}
		testingJava.value = true
		try {
			javaTestResult.value = await test_jre(path, version)
		} catch {
			javaTestResult.value = false
		}
		testingJava.value = false

		if (track) {
			trackEvent('JavaTest', { path, success: javaTestResult.value })
		}
	}

	function testJavaInstallationDebounced(path: string, version: number, delay = 600) {
		if (!path) return
		if (testDebounceTimer) clearTimeout(testDebounceTimer)
		testDebounceTimer = setTimeout(() => runJavaTest(path, version, false), delay)
	}

	async function testJavaInstallation(path: string, version: number, track = false) {
		if (!path) return
		if (testDebounceTimer) clearTimeout(testDebounceTimer)
		await runJavaTest(path, version, track)
	}

	return {
		testingJava,
		javaTestResult,
		testJavaInstallationDebounced,
		testJavaInstallation,
	}
}
