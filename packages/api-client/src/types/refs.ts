/**
 * Minimal ref-like structure to model both Vue refs and plain values uniformly.
 *
 * In Nuxt/Vue environments, handlers will return wrappers around real refs.
 * In non-Vue environments, this is a simple `{ value }` object.
 *
 * @template T Value type stored in the ref
 */
export interface SimpleRef<T> {
	value: T
}

export function refGetter<T>(get: () => T): SimpleRef<T> {
	return {
		get value() {
			return get()
		},
		set value(_: T) {
			// no-op; SimpleRef here is read-only proxy to Nuxt refs
		},
	} as unknown as SimpleRef<T>
}

export function ref<T>(value: T): SimpleRef<T> {
	return { value }
}
