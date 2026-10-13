import {type EventSource, EventStream} from "#ui/helpers/toolbox/misc";
import {
	type CounterCallbackData,
	logFactory,
	type Logger,
	type LoggerCallbackData,
	type LoggerFactory
} from '#ui/helpers/toolbox/misc/logging';

import {
	type ClientMessenger,
	type MessengerOptions,
	type ServerboundMessage,
	type InstanceConfig,
	type InstanceKey,
	isWorker,
	type Channel,
	type MessageTypes,
	type Messenger,
	type MessengerRegistry,
	type MessengerType,
	type MessengerVariants,
	type ClientboundMessage,
	type ServerMessageHandler,
	type ServerMessenger,
	type TypeId,
	type ServerboundChannels,
	type ServerHandlerResolver,
	type MaybeTransferable,
} from '.';

// #region MessengerType Types

type ChannelHolder = Worker | MessagePort;

type InternalMessenger<T extends MessageTypes | undefined = undefined, C extends ChannelHolder = ChannelHolder> = Messenger<T> & C & { terminate(): void; onTerminate: EventSource<OnTerminateCallback>; };

interface MessengerTypeImpl<T extends MessageTypes | undefined = undefined> extends MessengerType<T> {
    registry: MessengerRegistryImpl;

    instances?: (() => InternalMessenger<T>) | Map<InstanceKey, InternalMessenger<T>>;
    wrapWith: (channelHolder: () => ChannelHolder, key: InstanceKey) => InternalMessenger<T>;

    getOrCreateWith: (baseMessenger?: () => ChannelHolder, instanceKey?: string) => Messenger<T> & ChannelHolder;
    workerFactory?: () => Worker;

    getInstances: () => Messenger<T>[];
}

function typeFor<T extends MessageTypes>(
    id: string, instanceConfig: InstanceConfig, closeCallback?: (messenger: Messenger<T>) => void) {
    return {
        id: id,
        instances: instanceConfig.count == 1 ? undefined : new Map(),
        getInstances() {
            if (this.instances == null) return [];
            return (this.instances as Partial<Map<string, Messenger<T>>>).values ? (this.instances as Map<string, Messenger<T>>).values().toArray() : [(this.instances as unknown as Messenger<T>)];
        },
        getOrCreateWith(holder, instanceKey) {
            if ((instanceConfig.count == 1) && this.instances != null) {
                return (this.instances as () => Messenger<T>)();
            }

            if (holder == null) holder = this.workerFactory;

            if (instanceKey) {
                if (holder == null) throw new Error(`Unable to create messenger for '${this.id}' as the entry used was found to be null! Did you setup the main thread worker factory or pass in the port to the call!`)

				return this.wrapWith(holder, instanceKey);
            }

            if (holder == null) throw new Error(`Unable to create messenger for '${this.id}' as the entry used was found to be null! Did you setup the main thread worker factory or pass in the port to the call!`)

            let wrappedMessenger: InternalMessenger<T>;

            if (instanceConfig.count == 1) {
                wrappedMessenger = this.wrapWith(holder, "primary");

                wrappedMessenger.onTerminate.subscribe((forceClose) => {
                    if (forceClose || instanceConfig.timeout != null) this.instances = undefined as any;
                })

                this.instances = () => wrappedMessenger;
            } else {
                const instances = (this.instances as Map<InstanceKey, InternalMessenger<T>>);

                // TODO: ADD TIME BASED HASH?
                const key = `temp-${instances.size}`;

                wrappedMessenger = this.wrapWith(holder, key);

                (this.instances as Map<InstanceKey, Messenger<T>>).set(wrappedMessenger.key!, wrappedMessenger);
            }

            wrappedMessenger.postMessage?.({ channel: "set-key", key: wrappedMessenger.key} as SetOffThreadKeyMessage);

            return wrappedMessenger;
        },
        close(forceClosed) {
            if (instanceConfig.count == 1) {
                if (this.instances != undefined) {
                    const instance = (this.instances as () => InternalMessenger<T>)();
                    closeCallback?.(instance);
                    instance.postMessage({channel: "force-close-root", forceClose: forceClosed }  as ForceCloseMessengerRootMessage)
                }
            } else {
                const map = (this.instances as Map<InstanceKey, InternalMessenger<T>>);
                map.values().forEach(entry => {
                    closeCallback?.(entry)
                    entry.postMessage({channel: "force-close-root", forceClose: forceClosed } as ForceCloseMessengerRootMessage)
                })
                map.clear();
            }
        }
        /* wrapWith: wrapWith, */
    } as MessengerTypeImpl<T>;
}

// ---

interface MessengerRegistryImpl extends MessengerRegistry {
    serverMessenger?: ServerHandlerImpl<never>,
    types: Map<string, MessengerType<any>>;
    logger: LoggerFactory;
    onLogFactoryChange: EventSource<(logger: LoggerFactory) => void>;
}

export function createRegistryInternal(): MessengerRegistry {
    return {
        types: new Map(),
        logger: logFactory(),
		of: function <T extends MessageTypes, H>(id: string, extra: MessengerOptions<T, H> & InstanceConfig): MessengerType<T, H> {
            const instanceConfig: InstanceConfig = {
				timeout: extra?.timeout ?? undefined,
				shared: extra?.shared ?? true,
				count: extra.count ?? 1,
			};
            const type = typeFor<T>(id, instanceConfig);

            type.wrapWith = (holder, key) => setupAsClient(type, "client", key, extra.bypassServerMessageHandler != null ? {} as any : holder(), extra);

            type.registry = this;
            type.instanceConfig = instanceConfig;

            this.types.set(id, type);

            return type;
        },
        getType: function <T extends MessageTypes, H>(id: string): MessengerType<T, H> | undefined {
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
        getMessenger<T extends MessageTypes, H>(type: MessengerType<T, H>) {
            if (this.serverMessenger) {
                return this.serverMessenger.getMessenger(type);
            } else {
                return (type as MessengerTypeImpl<T>).getOrCreateWith();
            }
        },
        getActiveMessengers() {
            return this.types.values().flatMap((type) => (type as MessengerTypeImpl).getInstances()).toArray();
        },
		forActiveMessengersUnless(messenger: Messenger | undefined, callback: (connection: Messenger) => void) {
			for (const connection of this.getActiveMessengers()) {
				if (connection !== messenger) callback(connection);
			}
		},
		setWorkerFactoryFor(type, worker) {
			if (!isWorker()) (type as MessengerTypeImpl).workerFactory = worker;
        }
    } as MessengerRegistryImpl;
}

// ---

type OnTerminateCallback = (forceClose: boolean) => void

interface RootMessageHandler<MSG extends Channel<any> = Channel<any>> extends Messenger {
    handleRootMessage(msg: MSG, ports?: readonly MessagePort[]): Promise<boolean>;
}

type TaskHolder =  {
    hasTask: (transactionKey: string) => boolean,
    startTask: <T = unknown>() => ({ transactionKey: string, promise: Promise<T> }),
    endTask: (transactionKey: string, value: any, isError?: boolean) => boolean;
}

function createTaskHolder(): TaskHolder{
    const outstandingMessages: Map<string, PromiseWithResolvers<any>> = new Map();
    let index = 0;
    return {
        hasTask: (key) => outstandingMessages.has(key),
        startTask: <T>() => {
            const key = `${index++}`;

            const promiseHolder = Promise.withResolvers<T>();

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

interface ClientHandlerImpl<T extends MessageTypes, H = {}> extends RootMessageHandler<ClientboundMessengerChannels>, ClientMessenger<T, H> {
    taskHolder: TaskHolder;
}

const FORCED_OUTPUT_KEY = "INTERNAL_FORCED_OUTPUT";

function clientMessageHandler<T extends MessageTypes, H, C extends ChannelHolder>(
    messenger: InternalMessenger,
    options: MessengerOptions<T, H>,
): ClientHandlerImpl<T, H> & InternalMessenger<T, C> & (H extends object ? H : {}) {
    const taskHolder = createTaskHolder();

	async function handleInput<K extends ServerboundChannels<T>, M extends ServerboundMessage<T, K>>(msg: M) {
		if (options.bypassServerMessageHandler != null) {
			const output = await options.bypassServerMessageHandler!<K, M>(fullHandler as Messenger<T> as ServerMessenger<T>, msg, serverHandlerResolver<T>());
			messenger.terminate();
			messenger.isAlive = false;
			messenger.onTerminate.stream.sink(true)
			return (Array.isArray(output) ? output[0] : output)
		} else {
			const task = taskHolder.startTask<ClientboundMessage<T, K> extends never ? void : ClientboundMessage<T, K>>();
			const wrappedMsg: MessengerServerboundMessage<M> = {
				channel: "input", data: msg, key: task.transactionKey,
			};
			fullHandler.postMessage(wrappedMsg, options.serverMessageTransferHandler?.<K, M>(msg, (msg, resolvers) => (resolvers[msg.channel]?.(msg) ?? [])) ?? []);

			return await task.promise;
		}
	}

    const fullHandler = Object.assign(messenger, {
        taskHolder: taskHolder,
        async handleRootMessage(msg: ClientboundMessengerChannels){
            if (msg.channel == "operation-error" || msg.channel == "output") {
                handleOutputMessages(this, msg, messenger.logger, taskHolder, options.independentClientMessageHandler)
            } else if (messenger.variant == "client") {
                if (msg.channel == "logging") { // MAIN THREAD ONLY
                    const factory = messenger.logger.factory;
                    if (msg.logMsg.type == 'callback') {
                        factory.log(...msg.logMsg.data)
                    } else if (msg.logMsg.type == 'counter') {
                        factory.counter(...msg.logMsg.data)
                    } else {
                        throw new Error(`Unknown type for logging '${(msg.logMsg as Partial<{type: string}>).type}'!`)
                    }

                } else if (msg.channel == "setup-root") { // MAIN THREAD ONLY
                    const registry = messenger.type.registry;
                    const newMessenger = await registry.getMessenger<MessageTypes, C>(registry.getType(msg.id)!) as InternalMessenger<MessageTypes, C> & ClientMessenger<MessageTypes, unknown>;

                    const channel = new MessageChannel();

                    messenger.postMessage({ channel: "setup-port", id: newMessenger.type.id, key: newMessenger.key, transactionKey: msg.transactionKey } as SetupMessengerPortMessage, [channel.port1])
                    newMessenger.postMessage({ channel: "setup-port", id: messenger.type.id, key: messenger.key, transactionKey: msg.transactionKey } as SetupMessengerPortMessage, [channel.port2])
                } else if (msg.channel == "close-root") {
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
        sendMessage: handleInput,
    } as ClientHandlerImpl<T, H> & InternalMessenger<T, C> & (H extends object ? H : {}));

	fullHandler.handler = options.handlerConstructor(fullHandler);

	if (typeof fullHandler.handler == 'object') Object.assign(fullHandler, fullHandler.handler);

    return fullHandler;
}

function handleOutputMessages<T extends MessageTypes, H>(messenger: ClientHandlerImpl<T, H>, msg: MessengerOperationError | MessengerOutputMessage, logger: Logger, taskHolder: TaskHolder, forcedOutputHandler?: (messenger: ClientHandlerImpl<T, H>, msg: ClientboundMessage<T>) => boolean) {
    if (msg.channel == "operation-error") {
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

interface ServerHandlerImpl<T extends MessageTypes> extends RootMessageHandler<ServerboundMessengerChannels>, ServerMessenger<T>, Messenger {
    handleServerInput<K extends ServerboundChannels<T>, M extends ServerboundMessage<T, K>>(sender: InternalMessenger, msg: MessengerServerboundMessage<M>): Promise<void>;
    attemptTimeoutWait(): void;
    startShutDown(): void;
    setupPort(id: TypeId, transactionKey: string, key: InstanceKey, port: MessagePort): void;
    closePort(fullPortKey: string): void;
}

function serverMessageHandler<T extends MessageTypes>(
    messenger: InternalMessenger,
    onInput: ServerMessageHandler<T>
): ServerHandlerImpl<T> & InternalMessenger<T>  {
    let currentCloseFuncId: ReturnType<typeof setTimeout> | undefined = undefined;
    const currentInputKeys: Set<string> = new Set();

    const peerPorts: Map<string, Messenger<T> & MessagePort> = new Map();
    const waitingForPeerPort: Map<string, {type: MessengerType<any>, promiseHolder: PromiseWithResolvers<any>}> = new Map();

    const impl: ServerHandlerImpl<T> & InternalMessenger<T> = Object.assign(messenger, {
        async handleRootMessage(msg, ports) {
            if (msg.channel == "input") {
                this.handleServerInput(this, msg)
            } else if (this.variant == "server") {
                if (msg.channel == "set-key") {
                    this.key = msg.key;
                    this.logger = this.type.registry.getLogFactory().loggerFor(debugKeyFrom(this));
                } else if(msg.channel == "setup-port") {
                    this.setupPort(msg.id, msg.transactionKey, msg.key, ports![0])
                } else if(msg.channel == "force-close-root") {
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
        async handleServerInput<K extends ServerboundChannels<T>, M extends ServerboundMessage<T, K>>(sender: InternalMessenger<T>, msg: MessengerServerboundMessage<M>) {
            const transactionKey = `${fullInstanceKeyFrom(sender)}/${msg.key}`;

            currentInputKeys.add(transactionKey);

            const logger = this.logger;

            try {
                logger.debug(`Attempting to process '${msg.channel}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}' `, { obj: msg })

                const result = await onInput<K, M>(this, msg.data, serverHandlerResolver<T>(), sender);
                const tuple = (Array.isArray(result) ? result : [result, new Array<Transferable>()]) as [any, Transferable[]];

                logger.debug(`Finished processing '${msg.channel}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}'!`, { obj: msg })

                sender.postMessage(...([{channel: "output", data: tuple[0], key: msg.key} as MessengerOutputMessage, tuple[1]]))
            } catch (err) {
                logger.debug(`An error occurred when processing '${msg.channel}' on '${fullInstanceKeyFrom(this)}' for '${debugKeyFrom(sender)}'!`, { obj: msg })
                //logger.debug("Error has occurred on off-thread worker: ", { err: err, obj: msg})
                sender.postMessage({ channel: "operation-error", error: err, info: msg, key: msg.key} as MessengerOperationError)
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

            peerPorts.values().forEach((messenger) => messenger.postMessage({ channel: "close-port", id: messenger.type.id, key: messenger.key } as CloseMessengerPortMessage))

            setTimeout(() => {
                this.postMessage({ channel: "close-root", id: this.type.id, key: this.key } as CloseMessengerRootMessage)
            }, 100)
        },
        setupPort(id, transactionKey, key, port) {
            const fullTransactionKey = `${id}:${transactionKey}`;

            const data = waitingForPeerPort.get(fullTransactionKey)!;
            waitingForPeerPort.delete(fullTransactionKey);

            const portMessenger = ((data?.type ?? this.type.registry.getType(id)) as MessengerTypeImpl)
				.getOrCreateWith(() => port as Messenger & MessagePort, key) as InternalMessenger & RootMessageHandler & MessagePort;

            portMessenger.variant = "port"

            portMessenger.logger.debug(`Port '${fullInstanceKeyFrom(portMessenger)}' is being setup on '${fullInstanceKeyFrom(this)}'`)
            peerPorts.set(fullInstanceKeyFrom(portMessenger), portMessenger);

            const baseHandler = portMessenger.handleRootMessage.bind(portMessenger);
            const portClientHandler = portClientMessageHandler(this);

            portMessenger.handleRootMessage = async (msg, ports) => {
                if ((msg.channel == "close-port" || msg.channel == "output" || msg.channel == "operation-error") && await portClientHandler(msg as ClientboundPortMessages, ports)) {
                    return true;
                } else if (msg.channel == "input") {
                    this.handleServerInput(portMessenger, msg as MessengerServerboundMessage)
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

            this.postMessage({ channel: "setup-root", id: type.id, transactionKey: transactionKey} as StartMessengerRootMessage)

            return holder.promise as any;
        },
        getActiveMessengers() {
            return peerPorts.values().toArray() as Messenger<T>[];
        },
		forActiveMessengersUnless(messenger: Messenger<T> | undefined, callback: (connection: Messenger<T>) => void) {
			for (const connection of this.getActiveMessengers()) {
				if (connection !== messenger) callback(connection);
			}
		},
        async sendClientMessage(sender, msg) {
            const result = await msg();
            const tuple = (Array.isArray(result) ? result : [result, new Array<Transferable>()]) as [any, Transferable[]];

            (sender as InternalMessenger).postMessage(...([{channel: "output", data: tuple[0], key: FORCED_OUTPUT_KEY} as MessengerOutputMessage, tuple[1]]))
        }
    } as ServerHandlerImpl<T> & InternalMessenger<T>);

    // TODO: ADD WARNING TO PREVENT DOUBLE SERVER STUFF
    (impl.type.registry as MessengerRegistryImpl).serverMessenger = impl;

    return impl;
}

function portClientMessageHandler<T extends MessageTypes>(rootServerHandler: ServerHandlerImpl<T>): RootMessageHandler<ClientboundPortMessages>["handleRootMessage"] {
    return async (msg) => {
        if (msg.channel == "close-port") {

            rootServerHandler.closePort(fullInstanceKey(msg.id, msg.key));
            return true;
        }
        return false;
    }
}

// ---

export function setupHolderAsImpl<T extends MessageTypes, C extends ChannelHolder, R extends RootMessageHandler>(
    holder: C,
    type: MessengerTypeImpl<T>,
    variant: MessengerVariants,
    key: InstanceKey | undefined,
    rootHandlerSetup: (holder: C & InternalMessenger<T>) => C & InternalMessenger<T> & R
): InternalMessenger<T, C> & R {
    const infoHolder = holder as (C & InternalMessenger<T>);

    if (key) infoHolder.key = key;

    infoHolder.type = type as any;
    infoHolder.instanceData = type.instanceConfig;
    infoHolder.variant = variant;
    infoHolder.isAlive = true;

    const messenger = rootHandlerSetup(holder as C & InternalMessenger<T>);

	const close = (messenger as Partial<MessagePort>).close;

    if (close != null) {
		messenger.terminate = close.bind(messenger);
	} else if (messenger.terminate == null) {
		messenger.terminate = () => {};
	}

    const registry = type.registry;

    if (variant == "server" && registry.getLogFactory() == null) {
        registry.setLogFactory(logFactory({
            log: (level, loggerName, message, extra) => holder.postMessage({ channel: "logging", logMsg: { type: 'callback', data: [level, loggerName, message, extra]}} as MessengerLogMessage),
            counter: (level, loggerName, msg) => holder.postMessage({ channel: "logging", logMsg: { type: 'counter', data: [level, loggerName, msg]}} as MessengerLogMessage)
        }))
    }

    messenger.logger = registry.getLogFactory().loggerFor(debugKey(type.id, key, variant))

    messenger.onTerminate = new EventStream<OnTerminateCallback>((callbacks) => (forceClose) => callbacks.forEach((callback) => callback(forceClose))).source;

    messenger.onmessage = function (e: MessageEvent<Channel<string | MessengerLogMessage["channel"]>>) {
        const msg = e.data;

        if (msg.channel != "logging") messenger.logger.debug(`Message '${msg.channel}' received on '${debugKeyFrom(messenger)}'`, {obj: msg});

        messenger.handleRootMessage(msg, e.ports);
    };

    (messenger as Worker).onerror = (err: ErrorEvent) => {
        messenger.logger.error("An error occurred when attempting to complete a task and may be fatal error!", {err: err.error})
    };

    return messenger;
}

function setupAsClient<T extends MessageTypes, H, C extends ChannelHolder = ChannelHolder>(
    type: MessengerTypeImpl<T>,
    variant: MessengerVariants,
    key: InstanceKey,
    holder: C,
    options: MessengerOptions<T, H>,
): InternalMessenger<T, C> {
	return setupHolderAsImpl<T, C, ClientHandlerImpl<T, H>>(holder, type, variant, key, (impl) => clientMessageHandler(impl, options));
}

export function setupAsServerInternal<T extends MessageTypes>(
	type: MessengerType<T>,
	holder: ChannelHolder,
	inputHandler: ServerMessageHandler<T>,
): ServerMessenger<T> {
	return setupHolderAsImpl<T, ChannelHolder, ServerHandlerImpl<T>>(holder, type as MessengerTypeImpl<T>, "server", undefined,
		(impl) => serverMessageHandler<T>(impl, inputHandler));
}


// ---

type ClientboundMessengerChannels = StartMessengerRootMessage | MessengerOutputMessage | MessengerOperationError | MessengerLogMessage | CloseMessengerRootMessage;
type ServerboundMessengerChannels = MessengerServerboundMessage | SetupMessengerPortMessage | CloseMessengerPortMessage | SetOffThreadKeyMessage | ForceCloseMessengerRootMessage;

type ClientboundPortMessages = MessengerOperationError | MessengerOutputMessage | CloseMessengerPortMessage;

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

interface SetOffThreadKeyMessage extends Channel<"set-key"> {
    key: InstanceKey
}

interface StartMessengerRootMessage extends Channel<"setup-root">, MessengerTypeTarget {
    transactionKey: string,
}

interface SetupMessengerPortMessage extends Channel<"setup-port">, MessengerTarget {
    transactionKey: string,
}

interface CloseMessengerRootMessage extends Channel<"close-root">, MessengerTarget {}

interface ForceCloseMessengerRootMessage extends Channel<"force-close-root"> {
    forceClose: boolean
}

interface CloseMessengerPortMessage extends Channel<"close-port">, MessengerTarget {}

export interface MessengerServerboundMessage<T = any> extends Channel<"input"> {
    key: string;
    data: T;
}

interface MessengerLogMessage extends Channel<"logging"> {
    logMsg: {type: "callback", data: LoggerCallbackData} | { type: "counter", data: CounterCallbackData };
}

interface MessengerOutputMessage<T = any> extends Channel<"output">, ConversationTarget {
    data: T;
}

interface MessengerOperationError extends Channel<"operation-error">, ConversationTarget {
    error: any;
    info?: any;
}

// ---

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

function serverHandlerResolver<T extends MessageTypes>(): ServerHandlerResolver<T> {
	return async (msg, resolvers) => {
		const result = await resolvers[msg.channel]!(msg);

		return (
			typeof result === 'undefined'
				? undefined as void
				: (Array.isArray(result) ? [Object.assign(result[0], { channel: msg.channel }), result[1]] : Object.assign(result, { channel: msg.channel }))
		) as ClientboundMessage<T, typeof msg['channel']> extends never ? void : MaybeTransferable<ClientboundMessage<T, typeof msg['channel']>>
	}
}
