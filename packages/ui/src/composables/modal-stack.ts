const stack: symbol[] = []

export function useModalStack() {
	const id = Symbol()

	function push() {
		if (!stack.includes(id)) stack.push(id)
	}

	function pop() {
		const idx = stack.indexOf(id)
		if (idx !== -1) stack.splice(idx, 1)
	}

	function isTopmost() {
		return stack.length === 0 || stack[stack.length - 1] === id
	}

	function stackSize() {
		return stack.length
	}

	return { push, pop, isTopmost, stackSize }
}
