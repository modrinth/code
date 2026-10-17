import type { EventSource } from '#ui/helpers/toolbox/misc';
import { EventStream } from '#ui/helpers/toolbox/misc';

export type LogLevel = "debug" | "success" | "info" | "error" | "warning" | "neutral";

export function consoleOutFor(type: LogLevel) {
    if (type == 'error')        return console.error.bind(console);
    else if (type == 'warning') return console.warn.bind(console);
    else if (type == 'debug')   return console.debug.bind(console);
    else                        return console.info.bind(console);
}

export type ExtraData = {err?: Error, silence?: boolean, obj?: any | Array<any>, ext?: any}

export type LoggerCallbackData = [level: LogLevel, loggerName: string, message : string, extra?: ExtraData];
export type CounterCallbackData = [level: 'info' | 'debug', loggerName: string, message: {id: string, size?: number, amount?: number}];

export type LoggerCallback = (this: LoggerFactory, ...data: LoggerCallbackData) => void | Promise<void> | boolean;
export type CounterCallback = (this: LoggerFactory, ...data: CounterCallbackData) => void | Promise<void> | boolean;

export type LoggerEntryFormatter = (this: LoggerFactory, ...data: LoggerCallbackData) => string[];

export interface Counter {
    startCounting(id: string, size: number): void;
    incrementCount(id: string, amount: number): void;
}

export interface Timer {
    startTimer(id: string): void;

    endTimer(id: string): {start: Date, end: Date, totalTime: number} | undefined;
}

export function createTimer(): Timer {
    const map = new Map<string, Date>();

    return {
        startTimer: (id) => {
            map.delete(id);

            const startDate = new Date(Date.now());

            map.set(id, startDate);
        },
        endTimer: (id) => {
            const startDate = map.get(id);

            if (!startDate) return undefined;

            const endDate = new Date(Date.now());

            return {
                start: startDate,
                end: endDate,
                totalTime: endDate.getTime() - startDate.getTime()
            }
        }
    }
}

export function createCounter(): Counter {
    const map = new Map<string, { count: number, size: number }>();
    return {
        startCounting(id, size) {
            map.set(id, { count: 0, size: size })
        },
        incrementCount(id, amount) {
            const state = map.get(id);
            if (state) {
                state.count += amount;
            }
        }
    }
}

export interface LoggerFactory {
    log: LoggerCallback;
    formatter: LoggerEntryFormatter;
    counter: CounterCallback;

    onLog: EventSource<LoggerCallback>,
    onCounter: EventSource<CounterCallback>,

    loggerFor(name: string): Logger;

    startTimer(loggerName: string, id: string): void;

    endTimer(loggerName: string, id: string): {start: Date, end: Date, totalTime: number} | undefined;

    startCounting(loggerName: string, id: string, size: number): void;

    incrementCount(loggerName: string, id: string, amount: number): void;

    info(loggerName: string, message: string, extra?: ExtraData): void;

    success(loggerName: string, message: string, extra?: ExtraData): void;

    warn(loggerName: string, message: string, extra?: ExtraData): void;

    error(loggerName: string, message: string, extra?: ExtraData): void;

    debug(loggerName: string, message: string, extra?: ExtraData): void;
}

export class LoggerFactoryDelegate implements LoggerFactory {
    factory: LoggerFactory;

    constructor(factory: LoggerFactory) {
        this.log = factory.log;
        this.formatter = factory.formatter;
        this.counter = factory.counter;
        this.onLog = factory.onLog;
        this.onCounter = factory.onCounter;
        this.loggerFor = factory.loggerFor;
        this.startTimer = factory.startTimer;
        this.startCounting = factory.startCounting;
        this.incrementCount = factory.incrementCount;
        this.endTimer = factory.endTimer;
        this.info = factory.info;
        this.success = factory.success;
        this.warn = factory.warn;
        this.error = factory.error;
        this.debug = factory.debug;
        this.factory = factory;
    }

    log: LoggerCallback;

    formatter: LoggerEntryFormatter;

    counter: CounterCallback;

    onLog: EventSource<LoggerCallback>;

    onCounter: EventSource<CounterCallback>;

    loggerFor: (name: string) => Logger;

    startTimer: (loggerName: string, id: string) => void;

    endTimer: (loggerName: string, id: string) => {start: Date, end: Date, totalTime: number} | undefined;

    startCounting: (loggerName: string, id: string, size: number) => void;

    incrementCount: (loggerName: string, id: string, amount: number) => void;

    info: (loggerName: string, message: string, extra?: ExtraData) => void;

    success: (loggerName: string, message: string, extra?: ExtraData) => void;

    warn: (loggerName: string, message: string, extra?: ExtraData) => void;

    error: (loggerName: string, message: string, extra?: ExtraData) => void;

    debug: (loggerName: string, message: string, extra?: ExtraData) => void;
}

const defaultCallback: LoggerCallback = function (level, loggerName, message, extra) {
    const console = consoleOutFor(level);
    const entries: Array<any> = this.formatter(level, loggerName, message, extra);
    if (extra?.err) entries.push(extra.err);
    console(...entries);
    const debugInfo = extra?.obj;
    if (!debugInfo) return;
    // TODO: MAYBE USE SPREAD INSTEAD?
    if (Array.isArray(debugInfo)) {
        debugInfo.forEach(entry => console(entry))
    } else {
        console(debugInfo)
    }
}

const defaultCounter: CounterCallback = function (level, loggerName, message) {
    const key = `[${loggerName}]: ${message.id}`
    if (message.size) {
        consoleOutFor(level)(`[${loggerName}]: ${message.id} | Size ${message.size} `)
        console.countReset(key);
    } else if (message.amount) {
        for (let i = 0; i < message.amount; i++) {
            console.count(key)
        }
    }

}

const defaultFormatter: LoggerEntryFormatter = (_, loggerName, message) => [`[${loggerName}]:`, `${message}`];

export type FactoryOptions = {
    log?: LoggerCallback, formatter?: LoggerEntryFormatter, counter?: CounterCallback
}

// TODO: ADD ABILITY TO ADJUST WHAT LEVELS OR MESSAGES ARE LOGGED VIA EVENT?
export function logFactory({log, formatter, counter}: FactoryOptions = {}): LoggerFactory {
    const map = new Map<string, Map<string, Date>>();
    const factory: LoggerFactory =  {
        formatter: formatter ?? defaultFormatter,
        info(loggerName: string, message: string, extra: ExtraData) {
            this.log('info', loggerName, message, extra)
        },
        success(loggerName: string, message: string, extra: ExtraData) {
            this.log('success', loggerName, message, extra)
        },
        warn(loggerName: string, message: string, extra: ExtraData) {
            this.log('warning', loggerName, message, extra)
        },
        error(loggerName: string, message: string, extra: ExtraData) {
            this.log('error', loggerName, message, extra)
        },
        debug(loggerName: string, message: string, extra: ExtraData) {
            this.log('debug', loggerName, message,extra)
        },
        startTimer(loggerName: string, id: string) {
            const loggerEntries = (map.has(loggerName) ? map : map.set(loggerName, new Map()))
                .get(loggerName)!;

            loggerEntries.delete(id);

            const startDate = new Date(Date.now());

            loggerEntries.set(id, startDate);
        },
        endTimer(loggerName: string, id: string) {
            const loggerEntries = (map.has(loggerName) ? map : map.set(loggerName, new Map()))
                .get(loggerName)!;

            const startDate = loggerEntries.get(id);

            if (!startDate) return undefined;

            const endDate = new Date(Date.now());

            const timeData = {
                start: startDate,
                end: endDate,
                totalTime: endDate.getTime() - startDate.getTime()
            };

            this.debug(loggerName, `'${id}' took ${timeData.totalTime} ms to complete!`, { obj: timeData })

            return timeData;
        },
        startCounting(loggerName: string, id: string, size: number) {
            this.counter('info', loggerName, {id: id, size})
        },
        incrementCount(loggerName: string, id: string, amount: number) {
            this.counter('info', loggerName, {id: id, amount})
        },
        loggerFor(name: string): Logger {
            return {
                factory: this,
                name,
                startTimer(id: string) { this.factory.startTimer(this.name, id); },
                endTimer(id: string) { return this.factory.endTimer(this.name, id); },
                startCounting(id: string, size: number) { return this.factory.startCounting(this.name, id, size); },
                incrementCount(id: string, amount: number) { return this.factory.incrementCount(this.name, id, amount); },
                log(level, message, extra) { this.factory.log(level, name, message, extra ?? {}) },
                info(data, extra) { this.factory.info(this.name, data, extra) },
                success(data, extra) { this.factory.success(this.name, data, extra) },
                warn(data, extra) { this.factory.warn(this.name, data, extra) },
                error(data, extra) { this.factory.error(this.name, data, extra) },
                debug(data, extra) { this.factory.debug(this.name, data, extra) },
            }
        }
    } as LoggerFactory;

    const onLogEvent = new EventStream<LoggerCallback>((callbacks) => {
        const boundCallbacks = callbacks.map((callback) => callback.bind(factory));
        return (level, loggerName, message, extra) => {
            for (const callback of boundCallbacks) callback.bind(factory)(level, loggerName, message, extra);
        }
    }, factory);

    const finalCallback = (log ?? defaultCallback).bind(factory);
    factory.onLog = onLogEvent.source;
    factory.log =(level, loggerName, message, extra) => {
        finalCallback(level, loggerName, message, extra);
        // @ts-expect-error The 'this' context of type 'void' is not assignable to method's 'this' of type 'LoggerFactory'.
        onLogEvent.sink(level, loggerName, message, extra);
    }

    const onCounterEvent = new EventStream<CounterCallback>((callbacks) => {
        const boundCallbacks = callbacks.map((callback) => callback.bind(factory));
        return (level, loggerName, message) => {
            for (const callback of boundCallbacks) callback.bind(factory)(level, loggerName, message);
        }
    }, factory);

    const finalCounter = (counter ?? defaultCounter).bind(factory);

    factory.onCounter = onCounterEvent.source;
    factory.counter = (level, loggerName, message) => {
        finalCounter(level, loggerName, message);
        // @ts-expect-error The 'this' context of type 'void' is not assignable to method's 'this' of type 'LoggerFactory'.
        onCounterEvent.sink(level, loggerName, message);
    }
    return factory;
}

export interface Logger extends Timer, Counter {
    factory: LoggerFactory,
    name: string,

    log(level: LogLevel, message: string, extra?: ExtraData): void;

    info(message: string, extra?: ExtraData): void;

    success(message: string, extra?: ExtraData): void;

    warn(message: string, extra?: ExtraData): void;

    error(message: string, extra?: ExtraData): void;

    debug(message: string, extra?: ExtraData): void;
}
