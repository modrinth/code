const isClient = typeof window !== 'undefined'
const stack: symbol[] = []

export function useModalStack() {
	const id = Symbol()

	function push() {
		if (isClient && !stack.includes(id)) stack.push(id)
	}

	function pop() {
		if (!isClient) return
		const idx = stack.indexOf(id)
		if (idx !== -1) stack.splice(idx, 1)
	}

	function isTopmost() {
		if (!isClient) return true
		return stack.length === 0 || stack[stack.length - 1] === id
	}

	function stackSize() {
		return isClient ? stack.length : 0
	}

	return { push, pop, isTopmost, stackSize }
}
