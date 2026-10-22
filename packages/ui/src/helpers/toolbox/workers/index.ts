import DecompileWorker from './decompile?worker'
import { decompileMessengerType, messengerRegistry, zipHandlerType } from './registry'
import ZipWorker from './zip?worker'

export * from './registry'

messengerRegistry.setWorkerFactoryFor(decompileMessengerType, () => new DecompileWorker())
messengerRegistry.setWorkerFactoryFor(zipHandlerType, () => new ZipWorker())
