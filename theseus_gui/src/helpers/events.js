/*
    Event listeners for interacting with the Rust api
    These are all async functions that return a promise that resolves to the payload object (whatever Rust is trying to deliver)
*/

/*
    callback is a function that takes a single argument, which is the payload object (whatever Rust is trying to deliver)

*/
import { appWindow } from '@tauri-apps/api/window'

export async function loading_listener(callback) {
  return await appWindow.listen('loading', (event) => callback(event.payload))
}
