// #region Event Stream

/** Any listener an event stream can carry, whatever it takes and returns. */
export type EventCallback = (...args: never[]) => unknown

export class EventStream<T extends EventCallback> {
	protected sinkFactory: (callbacks: T[]) => T
	protected subscribers: T[] = []
	public source: EventSource<T>
	public sink: T
	private owner?: object

	constructor(sinkFactory: (callbacks: T[]) => T, owner?: object) {
		this.sinkFactory = sinkFactory
		this.sink = this.regenerateSink()
		this.source = new EventSource<T>(this)
		this.owner = owner
	}

	addSubscriber(subscriber: T): T {
		const boundFunc = this.owner ? (subscriber.bind(this.owner) as T) : subscriber
		this.subscribers.push(boundFunc)
		this.regenerateSink()
		return boundFunc
	}

	removeSubscriber(subscriber: T): void {
		this.subscribers = this.subscribers.filter((entry) => entry != subscriber)
		this.regenerateSink()
	}

	protected regenerateSink(): T {
		const sink = this.sinkFactory(this.subscribers)
		return (this.sink = this.owner ? (sink.bind(this.owner) as T) : sink)
	}
}

export class EventSource<T extends EventCallback> {
	stream: EventStream<T>

	constructor(stream: EventStream<T>) {
		this.stream = stream
	}

	public subscribe(subscriber: T): EventSubscription<T> {
		return new EventSubscription(this, this.stream.addSubscriber(subscriber))
	}
}

export class EventSubscription<T extends EventCallback> {
	protected source: EventSource<T>
	protected subscriber: T

	constructor(source: EventSource<T>, subscriber: T) {
		this.source = source
		this.subscriber = subscriber
	}

	public cancel(): void {
		this.source.stream.removeSubscriber(this.subscriber)
	}
}

// #endregion

// #region Observable Types

export type Consumer<T> = (value: T) => void

export class Observable<T> {
	static of<T>(startingValue: T) {
		let value = startingValue
		return new Observable<T>(
			() => value,
			(newValue) => (value = newValue),
		)
	}

	static and(...observables: Observable<boolean>[]) {
		const evalFunc = () => observables.every((o) => o.get())

		let value = evalFunc()
		const wrapper = new Observable<boolean>(() => value)

		const onChange = (_: boolean) => {
			const newValue = evalFunc()
			if (value != newValue) {
				value = newValue
				wrapper.onChangeEvent.sink(wrapper.get())
			}
		}
		observables.forEach((observable) => observable.onChange().subscribe(onChange))

		return wrapper
	}

	static or(...observables: Observable<boolean>[]) {
		const evalFunc = () => observables.find((o) => o.get())?.get() ?? false

		let value = evalFunc()
		const wrapper = new Observable(() => value)

		const onChange = (_: boolean) => {
			const newValue = evalFunc()
			if (value != newValue) {
				value = newValue
				wrapper.onChangeEvent.sink(wrapper.get())
			}
		}
		observables.forEach((observable) => observable.onChange().subscribe(onChange))

		return wrapper
	}

	get: () => T
	set: (value: T) => void
	protected onChangeEvent: EventStream<Consumer<T>>

	constructor(
		getCallback: () => T,
		setCallback?: (value: T) => T,
		runChangeCallbackOnSet?: boolean,
	) {
		this.get = getCallback
		this.set = (value) => {
			if (setCallback != null) setCallback(value)
			if (runChangeCallbackOnSet ?? true) this.onChangeEvent.sink(value)
		}
		this.onChangeEvent = new EventStream<Consumer<T>>(
			(callbacks) => (value: T) => callbacks.forEach((callback) => callback(value)),
		)
	}

	onChange(): EventSource<Consumer<T>> {
		return this.onChangeEvent.source
	}

	ifNotNull(func: Consumer<NonNullable<T>>) {
		const value = this.get()
		if (value != null) func(value)
	}

	mapTo<R>(convertTo: (t: T) => R, convertFrom?: (value: R) => T): Observable<R> {
		const mappedObs = new Observable(
			() => convertTo(this.get()),
			convertFrom
				? (value) => {
						this.set(convertFrom(value))
						return value
					}
				: undefined,
		)

		this.onChange().subscribe((value) => mappedObs.onChangeEvent.sink(convertTo(value)))

		return mappedObs
	}
}

export class KeyedObservable<T> extends Observable<T> {
	key: string

	constructor(
		key: string,
		getCallback: (key: string) => T,
		setCallback?: (value: T) => T,
		runChangeCallbackOnSet?: boolean,
	) {
		super(() => getCallback(this.key), setCallback, runChangeCallbackOnSet)
		this.key = key
	}
}

// #endregion

// #region Misc Util Methods

export function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms))
}

export function areArraysEqual(a: readonly unknown[], b: readonly unknown[]) {
	if (a === b) return true
	if (a == null || b == null || a.length !== b.length) return false

	for (let i = 0; i < a.length; ++i) {
		if (a[i] !== b[i]) return false
	}

	return true
}

// #endregion

// #region Lazy Values

export interface Supplier<T> {
	get: () => Promise<T>
}

export interface LazyValue<T> extends Supplier<T> {
	isSet(): boolean
	reset(): void
	lazyMap<R>(mapper: (value: T) => Promise<R>): LazyValue<R>
	map<R>(mapper: (value: T) => Promise<R>): Supplier<R>
}

export interface LazyMutableValue<T> extends LazyValue<T> {
	updateGetter(getter: () => Promise<T>): this
}

export function lazyMutableValue<T>(getter: () => Promise<T>): LazyMutableValue<T> {
	return Object.assign(lazyValue(getter), {
		updateGetter(getter: () => Promise<T>): LazyMutableValue<T> {
			this.reset()
			this._getter = getter
			return this
		},
	} as LazyMutableValueImpl<T>)
}

export function lazyValue<T>(getter: () => Promise<T>): LazyValue<T> {
	return {
		_getter: getter,
		_value: undefined,
		_isSet: false,

		isSet(): boolean {
			return this._isSet
		},

		async get(): Promise<T> {
			if (!this._isSet) {
				this._value = await this._getter()
				this._isSet = true
			}
			return this._value!
		},

		reset() {
			this._value = undefined
			this._isSet = false
			this._onResetEvent.stream.sink()
		},

		lazyMap<R>(mapper: (value: T) => R): LazyValue<R> {
			const value = lazyValue(async () => await mapper(await getter()))
			this._onResetEvent.subscribe(() => value.reset())
			return value
		},

		_onResetEvent: new EventStream<() => void>((callbacks) => () => {
			for (const callback of callbacks) {
				callback()
			}
		}).source,
	} as LazyValueImpl<T>
}

export interface LazyValueImpl<T> extends LazyValue<T> {
	_value: T | undefined
	_isSet: boolean
	_getter: () => Promise<T>
	_onResetEvent: EventSource<() => void>
}

export interface LazyMutableValueImpl<T> extends LazyValueImpl<T>, LazyMutableValue<T> {}

// #endregion

// #region Tree Node

export interface TreeNodeBase<N extends TreeNodeBase<N>> {
	parent?: N
	children: Map<string, N>
	name: string
	path: string

	sort(compare: (a: N, b: N) => number): this
}

export class TreeNode<N extends TreeNodeBase<N>> implements TreeNodeBase<N> {
	parent?: N
	children: Map<string, N> = new Map()
	name: string
	path: string

	constructor(parent: N | undefined, name: string, path: string) {
		this.parent = parent
		this.name = name
		this.path = path
	}

	sort(compare: (a: N, b: N) => number): this {
		this.children = new Map(
			Array.from(this.children.entries()).sort((entry1, entry2) => compare(entry1[1], entry2[1])),
		)
		for (const value of this.children.values()) value.sort(compare)
		return this
	}
}

// #endregion

export function wrapMap<K, VIn, VOut>(
	source: Map<K, VIn>,
	allowedKeys: Iterable<K>,
	mapper: (value: VIn, key: K) => VOut,
): ReadonlyMap<K, VOut> {
	return new MappedFilteredReadonlyMap(source, allowedKeys, mapper)
}

export class MappedFilteredReadonlyMap<K, VIn, VOut> implements ReadonlyMap<K, VOut> {
	private readonly allowedKeys: Set<K>

	constructor(
		private readonly sourceMap: ReadonlyMap<K, VIn>,
		allowedKeys: Iterable<K>,
		private readonly mapper: (value: VIn, key: K) => VOut,
	) {
		this.allowedKeys = new Set(allowedKeys)
	}

	has(key: K): boolean {
		return this.allowedKeys.has(key) && this.sourceMap.has(key)
	}

	get(key: K): VOut | undefined {
		if (!this.has(key)) return undefined
		const rawVal = this.sourceMap.get(key)
		return rawVal !== undefined ? this.mapper(rawVal, key) : undefined
	}

	get size(): number {
		let count = 0
		for (const key of this.allowedKeys) {
			if (this.sourceMap.has(key)) count++
		}
		return count
	}

	*entries(): MapIterator<[K, VOut]> {
		for (const [key, value] of this.sourceMap.entries()) {
			if (this.allowedKeys.has(key)) {
				yield [key, this.mapper(value, key)]
			}
		}
	}

	*keys(): MapIterator<K> {
		for (const key of this.allowedKeys) {
			if (this.sourceMap.has(key)) yield key
		}
	}

	*values(): MapIterator<VOut> {
		for (const [_, value] of this.entries()) {
			yield value
		}
	}

	forEach(
		callbackfn: (value: VOut, key: K, map: ReadonlyMap<K, VOut>) => void,
		thisArg?: unknown,
	): void {
		for (const [key, value] of this.entries()) {
			callbackfn.call(thisArg, value, key, this as ReadonlyMap<K, VOut>)
		}
	}

	[Symbol.iterator](): MapIterator<[K, VOut]> {
		return this.entries()
	}
}
