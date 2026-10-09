import { createRegistryInternal, setupAsServerInternal } from "#ui/helpers/toolbox/messenger/implement";
import type { Logger, LoggerFactory } from "#ui/helpers/toolbox/misc/logging";

export type TypeId = string;

/* @ts-ignore */
export interface MessengerType<M extends Messenger> {
    id: TypeId;
    registry: MessengerRegistry;
	instanceConfig: InstanceConfig;

    close(forceClosed?: boolean): void;
}

export interface MessengerAccessor {
    getMessenger: <M extends Messenger>(type: MessengerType<M>) => (M extends ClientMessenger<any, infer H> ? Promise<M & H> : Promise<M>) | undefined;
    getActiveMessengers: () => Messenger[];
}

export interface MessengerRegistry extends MessengerAccessor {
    of<T extends MessageTypes, H>(id: string, extra?: MessengerOptions<T, H> & EntryOptions<ClientMessenger<T, H>>): MessengerType<ClientMessenger<T, H>>;

    getType<M extends Messenger>(id: string): MessengerType<M> | undefined;

    setWorkerFactoryFor(type: MessengerType<Messenger>, factory: () => Worker): void;

    setLogFactory(factory: LoggerFactory): void;
    getLogFactory(): LoggerFactory;

    closeAll(forceClosed?: boolean): void;
}

export type EntryOptions<M extends Messenger> = { closeCallback?: (messenger: M) => void} & InstanceConfig;


//--

export type ValueBase = { [key: string]: any }

export interface Message<K extends string> { channel: K }

type IOType<I extends ValueBase = ValueBase, O extends ValueBase = ValueBase> = { input: I, output: O }

type MessageIOType<K extends string, I extends ValueBase = ValueBase, O extends ValueBase = ValueBase> = IOType<I, O> & Message<K>

export type MessageSchema = Record<string, IOType>;

export type MessageTypes<T extends MessageSchema = MessageSchema> =
    T extends Record<infer K extends string, IOType>
        ? { [E in AsString<K>]: T[E] extends IOType<infer I, infer O> ? MessageIOType<E, I, O> : never; }// & { '_type_flag_fix': MessageIOType<'_type_flag_fix', {}, {}>}
		: never;

export type AsString<T> = Extract<T, string>

export type InputMessage<T extends MessageTypes, K extends AsString<keyof T> = AsString<keyof T>> =
	T[K] extends MessageIOType<string, infer I> ? I & Message<K> : never;

export type OutputMessage<T extends MessageTypes, K extends AsString<keyof T> = AsString<keyof T>> =
	T[K] extends MessageIOType<string, object, infer O extends ValueBase> ? O & Message<K> : never;

export type OutputBase<T extends MessageTypes, K extends AsString<keyof T>>  =
	T[K] extends MessageIOType<string, object, infer O extends ValueBase> ? O : never;

export type MessageChannels<T extends MessageTypes> = keyof T;

// #endregion

// TODO: ATTEMPT TO ADD LOAD BALANCING FOR WHEN INSTANCES ARE SHARED AND COUNT IS GREATER THAN 1
export interface InstanceConfig {
    timeout?: number;
    shared: boolean;
	count: number;
}

export interface InstanceData {
	timeout?: number;
	shared: boolean;
}

export type InstanceKey = string;

export type MessengerVariants = "server" | "port" | "client";

export type ChannelHolder = Worker | MessagePort;

export interface Messenger {
    type: MessengerType<Messenger>;
    key?: InstanceKey;

    instanceData: InstanceData;
    variant: MessengerVariants;

    isAlive: boolean;
    logger: Logger;
}

export interface ServerMessenger<O extends ValueBase> extends Messenger, MessengerAccessor {
    /**
     * @experimental TBD if this should be exposed better
     */
    sendOutputMessage(sender: Messenger, msg: Promise<PossibleTransferable<O>>): Promise<void>;
}

export interface ClientMessenger<T extends MessageTypes, H> extends Messenger {

    type: MessengerType<ClientMessenger<T, H>>,

    sendMessage: <I_M extends InputMessage<T, K>, K extends AsString<keyof T>>(input: I_M) => Promise<OutputMessage<T, I_M['channel']>>

    handler: H;
}
export type MessengerHandlerConstructor<T extends MessageTypes, H> =  (messenger: ClientMessenger<T, H>) => H

//--

//--

export type PossibleTransferable<T> = T | [T, Transferable[]];

export type InputResolverMap<T extends MessageTypes> = {
	[E in AsString<keyof T>]?: ((msg: InputMessage<T, E>) => Transferable[]) | undefined
}

export type ClientInputUnpacker<T extends MessageTypes> =
	<I_M extends InputMessage<T>>(input: I_M) => Transferable[] | undefined;

// --

export type ClientOutputHandler<T extends MessageTypes, H> = (messenger: ClientMessenger<T, H>, output: OutputMessage<T>) => boolean;

export interface MessengerOptions<T extends MessageTypes, H> {
    handlerConstructor?: MessengerHandlerConstructor<T, H>
    inputBypassHandler?: ServerInputHandler<T>,
    inputTransferHandler?: ClientInputUnpacker<T>;
    independentOutputHandler?: ClientOutputHandler<T, H>;
}

export interface MessengerOptionsBase<T extends MessageTypes, H> {
	handlerConstructor?: MessengerHandlerConstructor<T, H>
	inputBypassHandler?: ServerInputHandler<T>,
	inputTransferHandler?: ClientInputUnpacker<T>;
	independentOutputHandler?: ClientOutputHandler<T, H>;
}

export function createRegistry(): MessengerRegistry {
    return createRegistryInternal();
}


// WORKER SECTION

export type ServerSimpleInputHandler<I extends ValueBase, O extends ValueBase> =
	(messenger: ServerMessenger<O>, input: I, sender?: Messenger) => Promise<PossibleTransferable<O>>;

export type ServerInputHandler<T extends MessageTypes> =
	<K extends AsString<keyof T>, I_M extends InputMessage<T, K>>(
		messenger: ServerMessenger<OutputMessage<T>>, input: I_M, sender?: Messenger
	) => Promise<PossibleTransferable<OutputMessage<T, K>>>

export type OutputResolverMap<T extends MessageTypes> = {
	[E in AsString<keyof T>]: (msg: InputMessage<T, E>) => Promise<PossibleTransferable<OutputBase<T, E>>>
}

export function setupAsServer<T extends MessageTypes>(
    type: MessengerType<Messenger>, holder: ChannelHolder, inputHandler: ServerInputHandler<T>
): ServerMessenger<OutputMessage<T>> {
    return setupAsServerInternal<T>(type, holder, inputHandler);
}

export function isWorker() {
	//@ts-expect-error TS2304: Cannot find name WorkerGlobalScope
	return typeof WorkerGlobalScope !== 'undefined' && self instanceof WorkerGlobalScope;
}

export async function resolveOutput<T extends MessageTypes, K extends AsString<keyof T>, I_M extends InputMessage<T, K>>(
	msg: I_M, resolvers: OutputResolverMap<T>
): Promise<PossibleTransferable<OutputMessage<T, K>>> {
	const result = await (resolvers[msg.channel]!(msg))

	return (Array.isArray(result) ? [Object.assign(result[0], { channel: msg.channel }), result[1]] : Object.assign(result, { channel: msg.channel })) as PossibleTransferable<OutputMessage<T, typeof msg['channel']>>
}

export function resolveInputTransfers<T extends MessageTypes, I_M extends InputMessage<T>>(
	msg: I_M, resolvers: InputResolverMap<T>
): Transferable[] | undefined {
	return resolvers[msg.channel]?.(msg);
}

// --
