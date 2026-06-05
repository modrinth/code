import { ref } from 'vue'

import { test_jre } from '@/helpers/jre.js'

export default function useJavaTest() {
	const testingJava = ref(false)
	const javaTestResult = ref<boolean | null>(null)
	let testDebounceTimer: ReturnType<typeof setTimeout> | null = null

	async function runJavaTest(path: string, version: number) {
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

	}

	function testJavaInstallationDebounced(path: string, version: number, delay = 600) {
		if (testDebounceTimer) clearTimeout(testDebounceTimer)
		if (!path) {
			javaTestResult.value = null
			return
		}
		testDebounceTimer = setTimeout(() => runJavaTest(path, version), delay)
	}

	async function testJavaInstallation(path: string, version: number, _track = false) {
		await runJavaTest(path, version)
	}

	return {
		testingJava,
		javaTestResult,
		testJavaInstallationDebounced,
		testJavaInstallation,
	}
}
