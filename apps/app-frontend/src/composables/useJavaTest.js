import { ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { test_jre } from '@/helpers/jre.js'

export default function useJavaTest() {
	const testingJava = ref(false)
	const javaTestResult = ref(null)
	let testDebounceTimer = null

	async function runJavaTest(path, version, track = true) {
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
			console.log('Tracking JavaTest event', { path, success: javaTestResult.value })
			trackEvent('JavaTest', { path, success: javaTestResult.value })
		}
	}

	function testJavaInstallationDebounced(path, version, delay = 600) {
		if (!path) return
		if (testDebounceTimer) clearTimeout(testDebounceTimer)
		testDebounceTimer = setTimeout(() => runJavaTest(path, version, false), delay)
	}

	async function testJavaInstallation(path, version, track = false) {
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
