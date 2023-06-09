/*
    Event listeners for interacting with the Rust api
    These are all async functions that return a promise that resolves to the payload object (whatever Rust is trying to deliver)
*/

/*
    callback is a function that takes a single argument, which is the payload object (whatever Rust is trying to deliver)

    You can call these to await any kind of emitted signal from Rust, and then do something with the payload object
    An example place to put this is at the start of main.js before the state is initialized- that way
    you can listen for any emitted signal from Rust and do something with it as the state is being initialized

    Example:
      import { loading_listener } from '@/helpers/events'
      await loading_listener((event) => {
        // event.event is the event name (useful if you want to use a single callback fn for multiple event types)
        // event.payload is the payload object
        console.log(event)
      })

    Putting that in a script will print any emitted signal from rust
*/
import { listen } from '@tauri-apps/api/event'

import {
  LoadingPayload,
  PayloadCallback,
  ProcessPayload,
  ProfilePayload,
  WarningPayload,
} from './payload'

export async function loading_listener<R>(callback: PayloadCallback<LoadingPayload, R>) {
  return await listen('loading', (event) => callback(event.payload as LoadingPayload))
}

export async function process_listener<R>(callback: PayloadCallback<ProcessPayload, R>) {
  return await listen('process', (event) => callback(event.payload as ProcessPayload))
}

export async function profile_listener<R>(callback: PayloadCallback<ProfilePayload, R>) {
  return await listen('profile', (event) => callback(event.payload as ProfilePayload))
}

export async function warning_listener<R>(callback: PayloadCallback<WarningPayload, R>) {
  return await listen('warning', (event) => callback(event.payload as WarningPayload))
}
