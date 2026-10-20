import {
	createRegistryInternal,
	setupAsServerInternal,
} from '#ui/helpers/toolbox/messenger/implement'
import type { Logger, LoggerFactory } from '#ui/helpers/toolbox/misc/logging'

/**
 * Messenger is an API used to set up communication between main thread and workers to in a manor
 * that allows for instantiation via a given registry on the main thread rather than on the workers
 * allowing for control better control. This means we can control the pool of workers and setup
 * worker <-> worker communication for existing workers allowing sharing of workers for variours
 * tasks.
 *
 * Communcation happens on a given Channel type either with a Serverbound (To the worker) Message
 * and/or Clientbound (To the Main thread) Message.
 */

//--

export type ValueBase = object | undefined

export type AsString<T> = Extract<T, string>

export interface ServerboundType<S extends ValueBase = ValueBase> {
	serverbound: S
}
export interface ClientboundType<C extends ValueBase = ValueBase> {
	clientbound: C
}

export type ClientServerBoundType<
	S extends ValueBase = ValueBase,
	C extends ValueBase = ValueBase,
> = ServerboundType<S> & ClientboundType<C>
export type ClientOrServerBoundType<
	S extends ValueBase = ValueBase,
	C extends ValueBase = ValueBase,
> = ServerboundType<S> | ClientboundType<C> | ClientServerBoundType<S, C>

export interface Channel<K extends string> {
	channel: K
}

export type ClientServerBoundChannel<
	K extends string,
	S extends ValueBase = ValueBase,
	C extends ValueBase = ValueBase,
> = Channel<K> & ClientServerBoundType<S, C>

export type ServerboundChannel<K extends string, S extends ValueBase = ValueBase> = Channel<K> &
	ServerboundType<S>
export type ClientboundChannel<K extends string, C extends ValueBase = ValueBase> = Channel<K> &
	ClientboundType<C>

export type MessageSchema = Record<string, ClientOrServerBoundType>

export type TypeToChannel<K extends string, T extends ClientOrServerBoundType> =
	T extends ClientServerBoundType<infer S, infer C>
		? ClientServerBoundChannel<K, S, C>
		: T extends ClientboundType<infer C>
			? ClientboundChannel<K, C>
			: T extends ServerboundType<infer S>
				? ServerboundChannel<K, S>
				: { type_error: 'NOT_VALID_TYPE' }

export type MessageTypes<T extends MessageSchema = MessageSchema> =
	T extends Record<string, ServerboundType | ClientboundType | ClientServerBoundType>
		? { [E in AsString<keyof T>]: TypeToChannel<E, T[E]> }
		: { type_error: 'INVALID_SCHEMA_TYPE' }

export type ServerboundChannels<T extends MessageTypes> = AsString<
	keyof { [E in AsString<keyof T> as ServerboundMessage<T, E> extends never ? never : E]: T[E] }
>

export type ClientboundChannels<T extends MessageTypes> = AsString<
	keyof { [E in AsString<keyof T> as ClientboundMessage<T, E> extends never ? never : E]: T[E] }
>

export type ClientServerBoundChannels<T extends MessageTypes> = AsString<
	keyof {
		[E in AsString<keyof T> as ServerboundMessage<T, E> & ClientboundMessage<T, E> extends never
			? never
			: E]: T[E]
	}
>

export type ServerboundMessage<
	T extends MessageTypes,
	K extends AsString<keyof T> = AsString<keyof T>,
> = T[K] extends ServerboundChannel<K, infer S> ? S & Channel<K> : never

export type ClientboundMessage<
	T extends MessageTypes,
	K extends AsString<keyof T> = AsString<keyof T>,
> = T[K] extends ClientboundChannel<K, infer C> ? C & Channel<K> : never

export type ClientboundBase<T extends MessageTypes, K extends AsString<keyof T>> =
	T[K] extends ClientServerBoundChannel<K, object, infer O extends ValueBase> ? O : never

// ---

export type TypeId = string

/** `_T` and `_H` aren't read here, but let callers infer a type's messages and handler from it. */
export interface MessengerType<_T extends MessageTypes | undefined = undefined, _H = undefined> {
	id: TypeId
	registry: MessengerRegistry
	instanceConfig: InstanceConfig

	close(forceClosed?: boolean): void
}

export interface MessengerAccessor<T extends MessageTypes | undefined = undefined> {
	getMessenger: <T extends MessageTypes, H>(
		type: MessengerType<T, H>,
	) => Promise<ClientMessenger<T, H> & H> | undefined
	getActiveMessengers(): Messenger<T>[]
}

export interface MessengerOptions<T extends MessageTypes, H> {
	handlerConstructor: MessengerHandlerConstructor<T, H>
	bypassServerMessageHandler?: ServerMessageHandler<T>
	serverMessageTransferHandler?: ServerMessageTransferHandler<T>
	independentClientMessageHandler?: ClientMessageHandler<T, H>
	closeCallback?: (messenger: ClientMessenger<T, H>) => void
}

// TODO: ATTEMPT TO ADD LOAD BALANCING FOR WHEN INSTANCES ARE SHARED AND COUNT IS GREATER THAN 1
export interface InstanceConfig {
	timeout?: number
	shared: boolean
	count: number
}

export interface MessengerRegistry extends MessengerAccessor {
	of<T extends MessageTypes, H>(
		id: string,
		extra?: MessengerOptions<T, H> & InstanceConfig,
	): MessengerType<T, H>

	getType<T extends MessageTypes>(id: string): MessengerType<T> | undefined

	setWorkerFactoryFor(type: MessengerType, factory: () => Worker): void

	setLogFactory(factory: LoggerFactory): void
	getLogFactory(): LoggerFactory

	closeAll(forceClosed?: boolean): void
}

//--

export interface InstanceData {
	timeout?: number
	shared: boolean
}

export type InstanceKey = string

export type MessengerVariants = 'server' | 'port' | 'client'

export interface Messenger<T extends MessageTypes | undefined = undefined> {
	type: MessengerType<T>
	key?: InstanceKey

	instanceData: InstanceData
	variant: MessengerVariants

	isAlive: boolean
	logger: Logger
}

export interface ServerMessenger<T extends MessageTypes>
	extends Messenger<T>, MessengerAccessor<T> {
	sendClientMessage(
		sender: Messenger<T>,
		msg: () => Promise<MaybeTransferable<ClientboundMessage<T>>>,
	): Promise<void>

	forConnections(
		callback: (connection: Messenger<T>) => void,
		except?: Messenger<T> | undefined,
	): void
}

export interface ClientMessenger<T extends MessageTypes, H> extends Messenger<T> {
	type: MessengerType<T, H>

	sendMessage: <K extends ServerboundChannels<T>, M extends ServerboundMessage<T, K>>(
		msg: M,
	) => Promise<
		ClientboundMessage<T, M['channel']> extends never ? void : ClientboundMessage<T, M['channel']>
	>

	handler: H
}

// --

export type MessengerHandlerConstructor<T extends MessageTypes, H> = (
	messenger: ClientMessenger<T, H>,
) => H

export type ClientMessageHandler<T extends MessageTypes, H> = (
	messenger: ClientMessenger<T, H>,
	output: ClientboundMessage<T>,
) => boolean

export type ServerMessageHandler<T extends MessageTypes> = <
	K extends ServerboundChannels<T>,
	M extends ServerboundMessage<T, K>,
>(
	messenger: ServerMessenger<T>,
	msg: M,
	resolver: ServerHandlerResolver<T>,
	sender?: Messenger<T>,
) => Promise<
	ClientboundMessage<T, K> extends never ? void : MaybeTransferable<ClientboundMessage<T, K>>
>

export type ServerHandlerResolver<T extends MessageTypes> = <
	K extends ServerboundChannels<T>,
	M extends ServerboundMessage<T, K>,
>(
	msg: M,
	handlers: ServerMessageHandlers<T>,
) => Promise<
	ClientboundMessage<T, K> extends never ? void : MaybeTransferable<ClientboundMessage<T, K>>
>

export type ServerMessageHandlers<T extends MessageTypes> = {
	[E in ServerboundChannels<T>]: (
		msg: ServerboundMessage<T, E>,
	) => Promise<
		ClientboundMessage<T, E> extends never ? void : MaybeTransferable<ClientboundBase<T, E>>
	>
}

export type MaybeTransferable<T> = T | [T, Transferable[]]

export type ServerMessageTransferHandler<T extends MessageTypes> = <
	K extends ServerboundChannels<T>,
	M extends ServerboundMessage<T, K>,
>(
	msg: M,
	resolver: (msg: M, handlers: ServerTransferHandlers<T>) => Transferable[],
) => Transferable[]

export type ServerTransferHandlers<T extends MessageTypes> = {
	[E in ServerboundChannels<T>]?: (msg: ServerboundMessage<T, E>) => Transferable[]
}

// --

export const createRegistry = createRegistryInternal

export const setupAsServer = setupAsServerInternal

export function isWorker() {
	//@ts-expect-error TS2304: Cannot find name WorkerGlobalScope
	return typeof WorkerGlobalScope !== 'undefined' && self instanceof WorkerGlobalScope
}
