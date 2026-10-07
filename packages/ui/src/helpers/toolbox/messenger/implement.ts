import { type EventSource,EventStream } from "#ui/helpers/toolbox/misc";
import { type CounterCallbackData, logFactory, type Logger, type LoggerCallbackData, type LoggerFactory } from '#ui/helpers/toolbox/misc/logging';

import {
	type ChannelHolder,
	type ClientCompoundMessenger,
	type ClientMessenger,
	type CompoundMessengerOptions,
	type EntryOptions,
	type InputMessageTypes,
type InstanceConfig,
	type InstanceKey,
	isWorker, 	type Message,
	type MessageSchema,
	type MessageTypes,
	type Messenger,
	type MessengerOptions,
	type MessengerRegistry,
	type MessengerType,
	type MessengerVariants,
	type OutputMessageTypes,
	type ServerInputHandler,
	type ServerMessenger,
	type TypeId} from '.';

// #region MessengerType Types

type InternalMessenger<M extends Messenger = Messenger, C extends ChannelHolder = ChannelHolder> = M & C & { terminate(): void; onTerminate: EventSource<OnTerminateCallback>; };

interface MessengerTypeImpl<M extends Messenger> extends MessengerType<M> {
    registry: MessengerRegistryImpl;

    instances?: (() => InternalMessenger<M>) | Map<InstanceKey, InternalMessenger<M>>;
    wrapWith: (channelHolder: () => ChannelHolder, key: InstanceKey) => InternalMessenger<M>;

    getOrCreateWith: (baseMessenger?: () => ChannelHolder, instanceKey?: string) => M extends ClientMessenger<any, any, infer H> ? M & ChannelHolder & H : M & ChannelHolder;
    workerFactory?: () => Worker;

    getInstances: () => M[];
}

type WrapperFunc<M extends Messenger> = <C extends ChannelHolder>(type: MessengerTypeImpl<M>, key: InstanceKey, holder: C) => M & C & InternalMessenger;

function typeFor<M extends Messenger>(
    id: string, instanceConfig: InstanceConfig, closeCallback?: (messenger: M) => void) {
    return {
        id: id,
        instances: instanceConfig.count == 1 ? undefined : new Map(),
        getInstances() {
            if (this.instances == null) return [];
            return (this.instances as Partial<Map<string, M>>).values ? (this.instances as Map<string, M>).values().toArray() : [(this.instances as unknown as M)];
        },
        getOrCreateWith(holder, instanceKey) {
            if ((instanceConfig.count == 1) && this.instances != null) {
                return (this.instances as () => M)();
            }

            if (holder == null) holder = this.workerFactory;

            if (instanceKey) {
                if (holder == null) throw new Error(`Unable to create messenger for '${this.id}' as the entry used was found to be null! Did you setup the main thread worker factory or pass in the port to the call!`)

                const wrappedMessenger = this.wrapWith(holder, instanceKey);

                return wrappedMessenger;
            }

            if (holder == null) throw new Error(`Unable to create messenger for '${this.id}' as the entry used was found to be null! Did you setup the main thread worker factory or pass in the port to the call!`)

            let wrappedMessenger: InternalMessenger<M>;

            if (instanceConfig.count == 1) {
                wrappedMessenger = this.wrapWith(holder, "primary");

                wrappedMessenger.onTerminate.subscribe((forceClose) => {
                    if (forceClose || instanceConfig.timeout != null) this.instances = undefined as any;
                })

                this.instances = () => wrappedMessenger;
            } else {
                const instances = (this.instances as Map<InstanceKey, InternalMessenger<M>>);

                // TODO: ADD TIME BASED HASH?
                const key = `temp-${instances.size}`;

                wrappedMessenger = this.wrapWith(holder, key);

                (this.instances as Map<InstanceKey, M>).set(wrappedMessenger.key!, wrappedMessenger);
            }

            wrappedMessenger.postMessage?.({ type: "set-key", key: wrappedMessenger.key} as SetOffThreadKeyMessage);

            return wrappedMessenger;
        },
        close(forceClosed) {
            if (instanceConfig.count == 1) {
                if (this.instances != undefined) {
                    const instance = (this.instances as () => InternalMessenger<M>)();
                    closeCallback?.(instance);
                    instance.postMessage({type: "force-close-root", forceClose: forceClosed }  as ForceCloseMessengerRootMessage)
                }
            } else {
                const map = (this.instances as Map<InstanceKey, InternalMessenger<M>>);
                map.values().forEach(entry => {
                    closeCallback?.(entry)
                    entry.postMessage({type: "force-close-root", forceClose: forceClosed } as ForceCloseMessengerRootMessage)
                })
                map.clear();
            }
        }
        /* wrapWith: wrapWith, */
    } as MessengerTypeImpl<M>;
}

// #endregion


// #region MessengerRegistry Types

interface MessengerRegistryImpl extends MessengerRegistry {
    setupMessengerType<I, O, H>(id: string, wrapWith: WrapperFunc<ClientMessenger<I, O, H>>, extra?: EntryOptions<ClientMessenger<I, O, H>>): MessengerType<ClientMessenger<I, O, H>>;

    serverMessenger?: ServerHandlerImpl,
    types: Map<string, MessengerType<any>>;
    logger: LoggerFactory;
    onLogFactoryChange: EventSource<(logger: LoggerFactory) => void>;
}

export function createRegistryInternal(): MessengerRegistry {
    return {
        types: new Map(),
        logger: logFactory(),
        compoundMessenger: function <T extends MessageSchema, H, P = MessageTypes<T>>(id: string, extra: CompoundMessengerOptions<T, H, P> & EntryOptions<ClientCompoundMessenger<T, H, P>>): MessengerType<ClientCompoundMessenger<T, H, P>> {
            function wrapWith<C extends ChannelHolder>(type: MessengerTypeImpl<ClientCompoundMessenger<T, H, P>>, key: string, channelHolder: C) {
                return setupAsCompoundClient(type, "client", key, channelHolder, extra)
            };

            // @ts-expect-error
            return this.setupMessengerType(id, wrapWith, extra);
        },
        simpleMessenger: function <I, O, H>(id: string, extra: MessengerOptions<I, O, H> & EntryOptions<ClientMessenger<I, O, H>>): MessengerType<ClientMessenger<I, O, H>> {
            function wrapWith<C extends ChannelHolder>(type: MessengerTypeImpl<ClientMessenger<I, O, H>>, key: string, channelHolder: C) {
                return setupAsClient(type, "client", key, channelHolder, extra)
            };

            // @ts-expect-error
            return this.setupMessengerType(id, wrapWith, extra);
        },
        setupMessengerType: function <I, O, H>(id: string, wrapWith: WrapperFunc<ClientMessenger<I, O, H>>, extra: MessengerOptions<I, O, H> & EntryOptions<Messenger>): MessengerType<ClientMessenger<I, O, H>> {
            const instanceConfig: InstanceConfig = {
				timeout: extra?.timeout ?? undefined,
				shared: extra?.shared ?? true,
				count: extra.count ?? 1,
			};
            const type = typeFor<ClientMessenger<I, O, H>>(id, instanceConfig);

            type.wrapWith = (holder, key) => wrapWith(type, key, extra.inputBypassHandler ? {} as any : holder());

            type.registry = this;
            type.instanceConfig = instanceConfig;

            this.types.set(id, type);

            return type;
        },
        getType: function <M extends Messenger>(id: string): MessengerType<M> | undefined {
            if (!this.types.has(id)) throw new Error(`MessengerRegistry did not contain entry '${id}'`);
            return this.types.get(id)!;
        },
        setLogFactory(logger: LoggerFactory): void {
            this.logger = logger;
        },
        getLogFactory(): LoggerFactory {
            return this.logger;
        },
        closeAll(forceClosed): void {
            this.types.values().forEach(async type => {
                type.close(forceClosed);
            })
        },
        getMessenger<M extends Messenger>(type: MessengerType<M>) {
            if (this.serverMessenger) {
                return this.serverMessenger.getMessenger(type);
            } else {
                return (type as MessengerTypeImpl<M>).getOrCreateWith();
            }
        },
        getActiveMessengers() {
            return this.types.values().flatMap((type) => (type as MessengerTypeImpl<any>).getInstances()).toArray();
        },
        setWorkerFactoryFor(type, worker) {
			if (!isWorker()) (type as MessengerTypeImpl<any>).workerFactory = worker;
        }
    } as MessengerRegistryImpl;
}

// #endregion


// ---

// #region Base Messenger Types

type OnTerminateCallback = (forceClose: boolean) => void

//#region Impl handling

function handleInput(messenger: Messenger & ChannelHolder, taskHolder: TaskHolder, input: any, transfer?: Transferable[]) {
    const task = taskHolder.startTask();

    messenger.postMessage({
        type: "input",
        data: input,
        key: task.transactionKey,
    } as MessengerInputMessage, transfer ?? []);

    return task.promise;
}

interface RootMessageHandler<MSG extends Message<any> = Message<any>> extends Messenger {
    handleRootMessage(msg: MSG, ports?: readonly MessagePort[]): Promise<boolean>;
}

type TaskHolder =  {
    hasTask: (transactionKey: string) => boolean,
    startTask: () => ({ transactionKey: string, promise: Promise<any> }),
    endTask: (transactionKey: string, value: any, isError?: boolean) => boolean;
}

function createTaskHolder(): TaskHolder{
    const outstandingMessages: Map<string, PromiseWithResolvers<any>> = new Map();
    let index = 0;
    return {
        hasTask: (key) => outstandingMessages.has(key),
        startTask: () => {
            const key = `${index++}`;

            const promiseHolder = Promise.withResolvers();

            outstandingMessages.set(key, promiseHolder);

            return  {transactionKey: key, promise: promiseHolder.promise };
        },
        endTask: (key, value, isError) => {
            const promise = outstandingMessages.get(key);
            if (promise == null) {
                return false;
            }

            outstandingMessages.delete(key);

            if (isError) {
                promise.reject(value);
            } else {
                promise.resolve(value);
            }

            return true;
        }
    }
}

interface ClientHandlerImpl<I = any, O = any, H = {}> extends RootMessageHandler<ClientboundMessages>, ClientMessenger<I, O, H> {
    taskHolder: TaskHolder;
}

const FORCED_OUTPUT_KEY = "INTERNAL_FORCED_OUTPUT";

function clientMessageHandler<I, O, H, C extends ChannelHolder>(
    messenger: InternalMessenger,
    options: MessengerOptions<I, O, H>,
): ClientHandlerImpl<I, O, H> & InternalMessenger<ClientHandlerImpl<I, O, H>, C> & (H extends Function ? {} : H) {
    const taskHolder = createTaskHolder();

    const sendMessage: ClientMessenger<I, O, H>["sendMessage"] = options.inputBypassHandler ? async (input) => {
        const output = (await options.inputBypassHandler!(messenger as any, input));
        messenger.terminate();
        messenger.isAlive = false;
        messenger.onTerminate.stream.sink(true)
        return Array.isArray(output) ? output[0] : output;
    } : (input) => handleInput(messenger, taskHolder, input, options.inputTransferHandler?.(input));

    const handler = options.handlerConstructor?.(messenger as any) ?? ({} as any)

    const fullHandler = Object.assign(messenger, {
        taskHolder: taskHolder,
        async handleRootMessage(msg: ClientboundMessages){
            if (msg.type == "operation-error" || msg.type == "output") {
                handleOutputMessages(this, msg, messenger.logger, taskHolder, options.forcedOutputHandler)
            } else if (messenger.variant == "client") {
                if (msg.type == "logging") { // MAIN THREAD ONLY
                    const factory = messenger.logger.factory;
                    if (msg.logMsg.type == 'callback') {
                        factory.log(...msg.logMsg.data)
                    } else if (msg.logMsg.type == 'counter') {
                        factory.counter(...msg.logMsg.data)
                    } else {
                        throw new Error(`Unknown type for logging '${(msg.logMsg as any).types}'!`)
                    }

                } else if (msg.type == "setup-root") { // MAIN THREAD ONLY
                    const registry = messenger.type.registry;
                    const newMessenger = await registry.getMessenger(registry.getType(msg.id)!) as InternalMessenger;

                    const channel = new MessageChannel();

                    messenger.postMessage({ type: "setup-port", id: newMessenger.type.id, key: newMessenger.key, transactionKey: msg.transactionKey } as SetupMessengerPortMessage, [channel.port1])
                    newMessenger.postMessage({ type: "setup-port", id: messenger.type.id, key: messenger.key, transactionKey: msg.transactionKey } as SetupMessengerPortMessage, [channel.port2])
                } else if (msg.type == "close-root") {
                    messenger.terminate();
                    messenger.isAlive = false;
                    messenger.onTerminate.stream.sink(true)
                } else {
                    return false;
                }
            } else {
                return false;
            }

            return true;
        },
        sendMessage: sendMessage,
        handler: handler,
    } as ClientHandlerImpl<I, O, H> & InternalMessenger);

    if (typeof handler === "object") Object.assign(fullHandler, handler);

    return fullHandler as any;
}

function handleOutputMessages<I, O, H>(messenger: ClientHandlerImpl<I, O, H>, msg: MessengerOperationError | MessengerOutputMessage<any>, logger: Logger, taskHolder: TaskHolder, forcedOutputHandler?: (messenger: ClientHandlerImpl<I, O, H>, msg: O) => boolean) {
    if (msg.type == "operation-error") {
        if (taskHolder.hasTask(msg.key)) {
            logger.error("An error occurred when attempting to complete a task!", {err: msg.error, obj: msg.info})
        } else {
            logger.error("An error occurred when attempting to task no longer present!", {err: msg.error, obj: msg.info})
        }

        taskHolder.endTask(msg.key, msg.error, true);
    } else { // MAIN THREAD OR PORT
        if (taskHolder.hasTask(msg.key)) {
            logger.debug("Completed a task for given data!", {obj: msg})
        } else {
            logger.error("Unable to Complete a task for given data as its not here anymore!!", {obj: msg})
        }

        if (msg.key == FORCED_OUTPUT_KEY) {
            if (forcedOutputHandler == null || !forcedOutputHandler(messenger, msg.data)) {
                logger.error("Unable to Complete a forced output for given data as its not supported for this messenger!", {obj: msg})
            }

            return;
        }

        taskHolder.endTask(msg.key, msg.data, false);
    }
}

interface ServerHandlerImpl<O = any> extends RootMessageHandler<ServerboundMessages>, ServerMessenger<O>, Messenger {
    handleServerInput(sender: InternalMessenger, msg: MessengerInputMessage): Promise<void>;
    attemptTimeoutWait(): void;
    startShutDown(): void;
    setupPort(id: TypeId, transactionKey: string, key: InstanceKey, port: MessagePort): void;
    closePort(fullPortKey: string): void;
}

function serverMessageHandler<O = any>(
    messenger: InternalMessenger,
    onInput: ServerInputHandler
): ServerHandlerImpl<O> & InternalMessenger  {
    let currentCloseFuncId: ReturnType<typeof setTimeout> | undefined = undefined;
    const currentInputKeys: Set<string> = new Set();

    const peerPorts: Map<string, Messenger & MessagePort> = new Map();
    const waitingForPeerPort: Map<string, {type: MessengerType<any>, promiseHolder: PromiseWithResolvers<any>}> = new Map();

    const impl: ServerHandlerImpl & InternalMessenger = Object.assign(messenger, {
        async handleRootMessage(msg, ports) {
            if (msg.type == "input") {
                this.handleServerInput(this, msg)
            } else if (this.variant == "server") {
                if (msg.type == "set-key") {
                    this.key = msg.key;
                    this.logger = this.type.registry.getLogFactory().loggerFor(debugKeyFrom(this));
                } else if(msg.type == "setup-port") {
                    this.setupPort(msg.id, msg.transactionKey, msg.key, ports![0])
                } else if(msg.type == "force-close-root") {
                    if (this.instanceData.timeout != null || msg.forceClose) {
                        this.startShutDown()
                    } else {
                        this.logger.warn("Close is either not supported on this messenger or it was not a force close!")
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }

            return true;
        },
        async handleServerInput(sender: InternalMessenger, msg: MessengerInputMessage) {
            const transactionKey = `${fullInstanceKeyFrom(sender)}/${msg.key}`;

            currentInputKeys.add(transactionKey);

            const logger = this.logger;

            try {
                logger.debug(`Attempting to process '${msg.type}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}' `, { obj: msg })

                const result = await onInput(this, msg.data, sender);
                const tuple = (Array.isArray(result) ? result : [result, new Array<Transferable>()]) as [any, Transferable[]];

                logger.debug(`Finished processing '${msg.type}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}'!`, { obj: msg })

                sender.postMessage(...([{type: "output", data: tuple[0], key: msg.key} as MessengerOutputMessage, tuple[1]]))
            } catch (err) {
                logger.debug(`An error occurred when processing '${msg.type}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}'!`, { obj: msg })
                //logger.debug("Error has occurred on off-thread worker: ", { err: err, obj: msg})
                sender.postMessage({ type: "operation-error", error: err, info: msg, key: msg.key} as MessengerOperationError)
            }

            currentInputKeys.delete(transactionKey);

            if (!this.instanceData.shared || (this.instanceData.timeout != null && currentInputKeys.size == 0)) {
                this.attemptTimeoutWait()
            }
        },
        attemptTimeoutWait() {
            if (!this.isAlive) return;
            const closeFuncId = setTimeout(async () => {
                //if (currentCloseFuncId != closeFuncId) return;

                if (!this.instanceData.shared || (this.instanceData.timeout != null && currentInputKeys.size == 0)) {
                    this.startShutDown();
                } else {
                    currentCloseFuncId = undefined;
                }
            }, this.instanceData.timeout ?? 100);
            clearTimeout(currentCloseFuncId);
            currentCloseFuncId = closeFuncId;
        },
        startShutDown() {
            if (!this.isAlive) return;

            this.isAlive = false;

            peerPorts.values().forEach((messenger) => messenger.postMessage({ type: "close-port", id: messenger.type.id, key: messenger.key } as CloseMessengerPortMessage))

            setTimeout(() => {
                this.postMessage({ type: "close-root", id: this.type.id, key: this.key } as CloseMessengerRootMessage)
            }, 100)
        },
        setupPort(id, transactionKey, key, port) {
            const fullTransactionKey = `${id}:${transactionKey}`;

            const data = waitingForPeerPort.get(fullTransactionKey)!;
            waitingForPeerPort.delete(fullTransactionKey);

            const portMessenger = ((data?.type ?? this.type.registry.getType(id)) as MessengerTypeImpl<Messenger>)!.getOrCreateWith(() => port as Messenger & MessagePort, key) as InternalMessenger<RootMessageHandler> & MessagePort;

            portMessenger.variant = "port"

            portMessenger.logger.debug(`Port '${fullInstanceKeyFrom(portMessenger)}' is being setup on '${fullInstanceKeyFrom(this)}'`)
            peerPorts.set(fullInstanceKeyFrom(portMessenger), portMessenger);

            const baseHandler = portMessenger.handleRootMessage.bind(portMessenger);
            const portClientHandler = portClientMessageHandler(this);

            portMessenger.handleRootMessage = async (msg, ports) => {
                if ((msg.type == "close-port" || msg.type == "output" || msg.type == "operation-error") && await portClientHandler(msg as ClientboundPortMessages, ports)) {
                    return true;
                } else if (msg.type == "input") {
                    this.handleServerInput(portMessenger, msg as MessengerInputMessage)
                }

                return baseHandler(msg, ports);
            }

            data?.promiseHolder?.resolve(portMessenger);
        },
        closePort(fullPortKey) {
            const portChannelHolder = peerPorts.get(fullPortKey);

            if (portChannelHolder != null) {
                peerPorts.delete(fullPortKey)
                portChannelHolder.close();
                this.logger.debug(`Port '${fullInstanceKeyFrom(portChannelHolder)}' is closed on '${fullInstanceKeyFrom(this)}'`);
            } else {
                this.logger.error(`Unable to close a given port as '${fullPortKey}' was not found on '${fullInstanceKeyFrom(this)}'`)
            }
        },
        getMessenger(type) {
            const transactionKey = `${peerPorts.size}`;

            const holder = Promise.withResolvers<any>();

            waitingForPeerPort.set(`${type.id}:${transactionKey}`, { type: type, promiseHolder: holder});

            this.postMessage({ type: "setup-root", id: type.id, transactionKey: transactionKey} as StartMessengerRootMessage)

            return holder.promise as any;
        },
        getActiveMessengers() {
            return peerPorts.values().toArray() as Messenger[];
        },
        async sendOutputMessage(sender, msg) {
            const result = await msg;
            const tuple = (Array.isArray(result) ? result : [result, new Array<Transferable>()]) as [any, Transferable[]];

            (sender as InternalMessenger).postMessage(...([{type: "output", data: tuple[0], key: FORCED_OUTPUT_KEY} as MessengerOutputMessage, tuple[1]]))
        }
    } as ServerHandlerImpl<O> & InternalMessenger);

    // TODO: ADD WARNING TO PREVENT DOUBLE SERVER STUFF
    (impl.type.registry as MessengerRegistryImpl).serverMessenger = impl;

    return impl;
}

function portClientMessageHandler(rootServerHandler: ServerHandlerImpl): RootMessageHandler<ClientboundPortMessages>["handleRootMessage"] {
    return async (msg) => {
        if (msg.type == "close-port") {

            rootServerHandler.closePort(fullInstanceKey(msg.id, msg.key));
            return true;
        }
        return false;
    }
}

//#endregion

// #region Base Messenger Constructors

export function setupHolderAsImpl<C extends ChannelHolder, R extends RootMessageHandler>(
    holder: C,
    type: MessengerTypeImpl<Messenger>,
    variant: MessengerVariants,
    key: InstanceKey | undefined,
    rootHandlerSetup: (holder: C & InternalMessenger) => C & InternalMessenger & R
): C & Messenger & R {
    const infoHolder = holder as (C & InternalMessenger);

    if (key) infoHolder.key = key;

    infoHolder.type = type as any;
    infoHolder.instanceData = type.instanceConfig;
    infoHolder.variant = variant;
    infoHolder.isAlive = true;

    const messenger = rootHandlerSetup(holder as C & InternalMessenger);

    if ((messenger as MessagePort).close) messenger.terminate = (messenger as MessagePort).close;

    const registry = type.registry;

    if (variant == "server" && registry.getLogFactory() == null) {
        registry.setLogFactory(logFactory({
            log: (level, loggerName, message, extra) => holder.postMessage({ type: "logging", logMsg: { type: 'callback', data: [level, loggerName, message, extra]}} as MessengerLogMessage),
            counter: (level, loggerName, msg) => holder.postMessage({ type: "logging", logMsg: { type: 'counter', data: [level, loggerName, msg]}} as MessengerLogMessage)
        }))
    }

    messenger.logger = registry.getLogFactory().loggerFor(debugKey(type.id, key, variant))

    messenger.onTerminate = new EventStream<OnTerminateCallback>((callbacks) => (forceClose) => callbacks.forEach((callback) => callback(forceClose))).source;

    messenger.onmessage = function (e: MessageEvent<Message<string | MessengerLogMessage["type"]>>) {
        const msg = e.data;

        if (msg.type != "logging") messenger.logger.debug(`Message '${msg.type}' received on '${debugKeyFrom(messenger)}'`, {obj: msg});

        messenger.handleRootMessage(msg, e.ports);
    };

    (messenger as Worker).onerror = (err: ErrorEvent) => {
        messenger.logger.error("An error occurred when attempting to complete a task and may be fatal error!", {err: err.error})
    };

    return messenger;
}

function setupAsCompoundClient<T extends MessageSchema, H, P = MessageTypes<T>>(
    type: MessengerTypeImpl<Messenger>,
    variant: MessengerVariants,
    key: InstanceKey,
    holder: ChannelHolder,
    options: CompoundMessengerOptions<T, H, P>,
): ClientCompoundMessenger<T, H, P> {
    // @ts-expect-error
    const extendOptions: MessengerOptions<InputMessageTypes<T, P>, OutputMessageTypes<T, P>, H> = {
        ...options,
        inputTransferHandler: options?.inputTransferHandler == null ? undefined : (msg) => {
            const copied = {
                ...msg,
                resolve: (resolver: any) => resolver[msg.type]?.(msg) ?? []
            }
            return options.inputTransferHandler!(copied as any);
        }
    };
    // @ts-expect-error
    return setupAsClient<InputMessageTypes<T, P>, OutputMessageTypes<T, P>, H>(type, variant, key, holder, extendOptions) as ClientCompoundMessenger<T, H, P>;
}

function setupAsClient<I, O, H = {}, C extends ChannelHolder = ChannelHolder>(
    type: MessengerTypeImpl<Messenger>,
    variant: MessengerVariants,
    key: InstanceKey,
    holder: C,
    options: MessengerOptions<I, O, H>,
): ClientHandlerImpl<I, O, H> {
    return setupHolderAsImpl<C, ClientHandlerImpl<I, O, H>>(holder, type, variant, key, (impl) => clientMessageHandler(impl, options));
}

export function setupAsServerInternal<I, O>(
    type: MessengerType<Messenger>,
    holder: ChannelHolder,
    inputHandler: ServerInputHandler<I, O>,
): ServerMessenger {
    return setupHolderAsImpl<ChannelHolder, ServerHandlerImpl>(holder, type as MessengerTypeImpl<Messenger>, "server", undefined, (impl) => serverMessageHandler<O>(impl, inputHandler));
}


// #endregion


// #region Root Messenger Message types

type ClientboundMessages = StartMessengerRootMessage | MessengerOutputMessage | MessengerOperationError | MessengerLogMessage | CloseMessengerRootMessage;
type ServerboundMessages = MessengerInputMessage | SetupMessengerPortMessage | CloseMessengerPortMessage | SetOffThreadKeyMessage | ForceCloseMessengerRootMessage;

type ClientboundPortMessages = MessengerOperationError | MessengerOutputMessage<any> | CloseMessengerPortMessage;

interface MessengerTarget extends MessengerTypeTarget {
    id: TypeId;
    key: InstanceKey;
}

interface MessengerTypeTarget {
    id: TypeId;
}

type ConversationId = string;

interface ConversationTarget {
    key: ConversationId;
}

interface SetOffThreadKeyMessage extends Message<"set-key"> {
    key: InstanceKey
}

interface StartMessengerRootMessage extends Message<"setup-root">, MessengerTypeTarget {
    transactionKey: string,
}

interface SetupMessengerPortMessage extends Message<"setup-port">, MessengerTarget {
    transactionKey: string,
}

interface CloseMessengerRootMessage extends Message<"close-root">, MessengerTarget {}

interface ForceCloseMessengerRootMessage extends Message<"force-close-root"> {
    forceClose: boolean
}

interface CloseMessengerPortMessage extends Message<"close-port">, MessengerTarget {}

export interface MessengerInputMessage<T = any> extends Message<"input"> {
    key: string;
    data: T;
}

interface MessengerLogMessage extends Message<"logging"> {
    logMsg: {type: "callback", data: LoggerCallbackData} | { type: "counter", data: CounterCallbackData };
}

interface MessengerOutputMessage<T = any> extends Message<"output">, ConversationTarget {
    data: T;
}

interface MessengerOperationError extends Message<"operation-error">, ConversationTarget {
    error: any;
    info?: any;
}

// #endregion

function fullInstanceKeyFrom(messenger: Messenger) {
    return fullInstanceKey(messenger.type.id, messenger.key);
}

function fullInstanceKey(id: TypeId, key: InstanceKey | undefined) {
    return `${id}:${key ?? "UNSET"}`
}

function debugKeyFrom(messenger: Messenger) {
    return debugKey(messenger.type.id, messenger.key, messenger.variant);
}

function debugKey(id: TypeId, key: InstanceKey | undefined, variant: MessengerVariants) {
    return `${id}:${key ?? "UNSET"}#${variant}`
}
