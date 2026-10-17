import { onScopeDispose, shallowRef, watch } from 'vue'

export type WorkbenchWindows = ReturnType<typeof useWorkbenchWindows>

/**
 * The windows showing the workbench: the main one and any popouts. Popout windows share the main
 * window's JavaScript, so listeners meant for the whole workbench go on each of them.
 */
export function useWorkbenchWindows() {
	const windows = shallowRef<Window[]>(typeof window === 'undefined' ? [] : [window])

	/** Listens on every window showing the workbench, as they come and go, until stopped. */
	function listen<K extends keyof WindowEventMap>(
		event: K,
		listener: (event: WindowEventMap[K]) => void,
		options?: AddEventListenerOptions,
	) {
		const stop = watch(
			windows,
			(targets, _, onCleanup) => {
				for (const target of targets) target.addEventListener(event, listener, options)
				onCleanup(() => {
					for (const target of targets) target.removeEventListener(event, listener, options)
				})
			},
			{ immediate: true, flush: 'sync' },
		)
		onScopeDispose(stop)
		return stop
	}

	function register(target: Window) {
		windows.value = [...windows.value, target]
		return () => {
			windows.value = windows.value.filter((entry) => entry !== target)
		}
	}

	return { windows, listen, register }
}
