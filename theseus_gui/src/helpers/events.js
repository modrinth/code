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

/// Payload for the 'loading' event
/*
    LoadingPayload {
        event: {
          type: string, one of  "StateInit", "PackDownload", etc
          (Optional fields depending on event type)
          pack_name: name of the pack
          pack_id, optional, the id of the modpack
          pack_version, optional, the version of the modpack
          profile_name: name of the profile
          profile_uuid: unique identification of the profile

        }
        loader_uuid: unique identification of the loading bar
        fraction: number, (as a fraction of 1, how much we've loaded so far). If null, by convention, loading is finished
        message: message to display to the user
    }
*/
export async function loading_listener(callback) {
  return await listen('loading', (event) => callback(event.payload))
}

/// Payload for the 'process' event
/*
    ProcessPayload {
        uuid: unique identification of the process in the state (currently identified by PID, but that will change)
        pid: process ID
        event: event type ("Launched", "Finished")
        message: message to display to the user
    }
*/
export async function process_listener(callback) {
  return await listen('process', (event) => callback(event.payload))
}

/// Payload for the 'profile' event
/*
    ProfilePayload {
        uuid: unique identification of the process in the state (currently identified by path, but that will change)
        name: name of the profile
        path: path to profile
        event: event type ("Created", "Added", "Edited", "Removed")
    }
*/
export async function profile_listener(callback) {
  return await listen('profile', (event) => callback(event.payload))
}

/// Payload for the 'warning' event
/*
    WarningPayload {
        message: message to display to the user
    }
*/
export async function warning_listener(callback) {
  return await listen('warning', (event) => callback(event.payload))
}
