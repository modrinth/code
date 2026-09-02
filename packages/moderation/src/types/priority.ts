// Very cool Priority system I made -chyz
export class Priority {
	private _beforeCount = 0
	private _afterCount = 0

	/** @internal */
	constructor(
		private readonly _anchor: Priority | null = null,
		private readonly _direction: -1 | 1 = 1,
		private readonly _index: number = 0,
	) {}

	before(): Priority {
		return new Priority(this, -1, ++this._beforeCount)
	}
	after(): Priority {
		return new Priority(this, 1, ++this._afterCount)
	}

	compareTo(other: Priority): number {
		const ra = this._resolve()
		const rb = other._resolve()
		for (let i = 0; i < Math.max(ra.length, rb.length); i++) {
			const diff = (ra[i] ?? 0) - (rb[i] ?? 0)
			if (diff !== 0) return diff
		}
		return 0
	}

	private _resolve(): number[] {
		if (this._anchor === null) return [0]
		return [...this._anchor._resolve(), this._direction, this._index]
	}
}
