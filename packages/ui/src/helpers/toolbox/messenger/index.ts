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
    getMessenger: <M extends Messenger>(type: MessengerType<M>) => (M extends ClientMessenger<any, any, infer H> ? Promise<M & H> : Promise<M>) | undefined;
    getActiveMessengers: () => Messenger[];
}

export interface MessengerRegistry extends MessengerAccessor {
    compoundMessenger<T extends MessageSchema, H, P = MessageTypes<T>>(id: string, extra?: CompoundMessengerOptions<T, H, P> & EntryOptions<ClientCompoundMessenger<T, H, P>>): MessengerType<ClientCompoundMessenger<T, H, P>>;

    simpleMessenger<I, O, H>(id: string, extra?: MessengerOptions<I, O, H> & EntryOptions<ClientMessenger<I, O, H>>): MessengerType<ClientMessenger<I, O, H>>;

    getType<M extends Messenger>(id: string): MessengerType<M> | undefined;

    setWorkerFactoryFor(type: MessengerType<Messenger>, factory: () => Worker): void;

    setLogFactory(factory: LoggerFactory): void;
    getLogFactory(): LoggerFactory;

    closeAll(forceClosed?: boolean): void;
}

export type EntryOptions<M extends Messenger> = { closeCallback?: (messenger: M) => void} & InstanceConfig;


//--

export interface Message<K extends string> { type: K }

// #region Compound Message Types

export type MessageSchema = Record<string, {input: any, output: any}>;

export type MessageTypes<T extends MessageSchema> =
    T extends Record<infer K extends string, { input: any, output: any}>
        ? { [E in K]: {type: E, input: T[E]["input"] /* & Message<E> */ /* & O */, output: T[E]["output"] /* & Message<E> */ /* & I */}; }
        : never;

export type InputMessageTypes<T extends MessageSchema, P = MessageTypes<T>> = P[Extract<keyof P, string>] extends { type: infer K extends string, input: infer I } ? I & Message<K> : never;
export type OutputMessageTypes<T extends MessageSchema, P = MessageTypes<T>> = P[Extract<keyof P, string>] extends { type: infer K extends string, output: infer O } ? O & Message<K> : never;
export type TypeMessageTypes<T extends MessageSchema, P = MessageTypes<T>> = P[Extract<keyof P, string>] extends { type: infer T } ? T : never;

export type InputMessageFor<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = P[K] extends { input: infer I } ? I & Message<K> : never;
export type OutputMessageFor<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = P[K] extends { output: infer O } ? O & Message<K> : never;

export type InputFor<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = P[K] extends { input: infer I } ? I : never;
export type OutputFor<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = P[K] extends { output: infer O } ? O : never;

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

export type InstanceTypes = "singleton-shared" | "temporary-exclusive" | "temporary-shared";

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

export interface ServerMessenger<O = any> extends Messenger, MessengerAccessor {
    /**
     * @experimental TBD if this should be exposed better
     */
    sendOutputMessage(sender: Messenger, msg: Promise<PossibleTransferable<O>>): Promise<void>;
}

export interface ClientMessenger<I = any, O = any, H = never> extends Messenger  {
    type: MessengerType<ClientMessenger<I, O, H>>,

    sendMessage: (input: I) => Promise<O>;

    handler: H;
}


// @ts-expect-error - No worries as we are stating the the input will always have a tied to output but TS can't seem to resolve it for some reason
export interface ClientCompoundMessenger<T extends MessageSchema, H, P = MessageTypes<T>> extends ClientMessenger<InputMessageTypes<T, P>, OutputMessageTypes<T,P>, H>/* , CompoundInputHandler<T, P> */ {
    type: MessengerType<ClientCompoundMessenger<T, H, P>>,

    sendMessage: <I_M extends InputMessageFor<T, K, P>, K extends Extract<keyof P, string>>(input: I_M) => Promise<OutputMessageFor<T, I_M["type"], P>>

    handler: H;
}

export type MessengerHandlerConstructor<I, O, H = {}> = (messenger: ClientMessenger<I, O, H>) => H

export type CompoundHandlerConstructor<T extends MessageSchema, H, P = MessageTypes<T>> =  (messenger: ClientCompoundMessenger<T, H, P>) => H

//--

//--


export type PossibleTransferable<T> = T | [T, Transferable[]];

export type ClientInputUnpacker<I = any> = (input: I) => Transferable[] | undefined;


export interface CompoundInputClientMessageExt<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> extends Message<K> {
    resolve(handlers: InputResolverMap<T, P>): Transferable[] | undefined;
}

export type InputResolverMap<T extends MessageSchema, P = MessageTypes<T>> = { [E in Extract<keyof P, string>]?: ((msg: InputMessageFor<T, E, P>) => Transferable[]) | undefined}

export type CompoundInputClientMessage<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = CompoundInputClientMessageExt<T, K, P> & InputMessageFor<T, K, P>

export type ClientCompoundInputUnpacker<T extends MessageSchema, P = MessageTypes<T>>
    = <I_M extends CompoundInputClientMessage<T, K, P>, K extends Extract<keyof P, string>>(input: I_M) => Transferable[] | undefined;

// --

export interface MessengerOptions<I = any, O = any, H = {}> {
    handlerConstructor?: MessengerHandlerConstructor<I, O, H>,
    inputBypassHandler?: ServerInputHandler<I, O>,
    inputTransferHandler?: ClientInputUnpacker<I>;
    forcedOutputHandler?: (messenger: ClientMessenger<I, O, H>, output: O) => boolean;
}

// @ts-expect-error - No worries as we are stating the the input will always have a tied to output but TS can't seem to resolve it for some reason
export interface CompoundMessengerOptions<T extends MessageSchema, H, P = MessageTypes<T>> extends MessengerOptions<InputMessageTypes<T, P>, OutputMessageTypes<T,P>, H> {
    handlerConstructor?: CompoundHandlerConstructor<T, H, P>
    inputBypassHandler?: ServerCompoundInputHandler<T, P>,
    inputTransferHandler?: ClientCompoundInputUnpacker<T, P>;
    forcedOutputHandler?: (messenger: ClientCompoundMessenger<T, H, P>, output: OutputMessageTypes<T, P>) => boolean;
}

export function createRegistry(): MessengerRegistry {
    return createRegistryInternal();
}


// WORKER SECTION

export type ServerInputHandler<I = any, O = any> = (messenger: ServerMessenger<O>, input: I, sender?: Messenger) => Promise<PossibleTransferable<O>>;


export type ServerCompoundInputHandler<T extends MessageSchema, P = MessageTypes<T>>
    = <I_M extends CompoundInputServerMessage<T, K, P>, K extends Extract<keyof P, string>>(messenger: ServerMessenger<OutputMessageTypes<T, P>>, input: I_M, sender?: Messenger) => Promise<PossibleTransferable<OutputMessageFor<T, K, P>>>


export interface CompoundInputServerMessageExt<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> extends Message<K> {
    resolve(handlers: OutputResolverMap<T, P>): Promise<OutputMessageFor<T, K, P>>;
}

export type OutputResolverMap<T extends MessageSchema, P = MessageTypes<T>> = { [E in Extract<keyof P, string>]: (msg: InputMessageFor<T, E, P>) => Promise<PossibleTransferable<OutputFor<T, E, P>>>}

export type CompoundInputServerMessage<T extends MessageSchema, K extends Extract<keyof P, string>, P = MessageTypes<T>> = CompoundInputServerMessageExt<T, K, P> & InputMessageFor<T, K, P>;


export function setupAsCompoundServer<T extends MessageSchema, P = MessageTypes<T>>(
    type: MessengerType<Messenger>,
    holder: ChannelHolder,
    inputHandler: ServerCompoundInputHandler<T, P>
): ServerMessenger {
    // @ts-expect-error
    return setupAsServer<InputMessageTypes<T, P>, OutputMessageTypes<T, P>>(type, holder, async (messenger, msg, sender) => {
        const copiedMsg = {
            ...msg,
            resolve: (resolvers: any) => {
                const result = (resolvers[msg.type] as ((msg: any) => any) | undefined)!(msg) as PossibleTransferable<object>;

                return (Array.isArray(result)) ? [Object.assign(result[0], { type: msg.type }), result[1]] : Object.assign(result, { type: msg.type })
            }
        } as any;
        return inputHandler(messenger, copiedMsg, sender);
    });
}

export function setupAsServer<I, O>(
    type: MessengerType<Messenger>,
    holder: ChannelHolder,
    inputHandler: ServerInputHandler<I, O>,
): ServerMessenger {
    return setupAsServerInternal(type, holder, inputHandler);
}

export function isWorker() {
	//@ts-expect-error TS2304: Cannot find name WorkerGlobalScope
	return typeof WorkerGlobalScope !== 'undefined' && self instanceof WorkerGlobalScope;
}

// --
