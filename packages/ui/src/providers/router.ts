// import { createContext } from '.'

// export interface RouteLocationRaw {
//   path?: string
//   name?: string
//   params?: Record<string, any>
//   query?: Record<string, any>
// }

// export interface RouteLocationNormalized {
//   path: string
//   name?: string | null
//   params: Record<string, any>
//   query: Record<string, any>
//   hash: string
//   fullPath: string
// }

// export interface RouteRecordRaw {
//   path: string
//   name?: string
//   component: any
//   meta?: Record<string, any>
//   children?: RouteRecordRaw[]
//   redirect?: string | RouteLocationRaw
// }

// export abstract class AbstractRouterManager {
//   /**
//    * Navigate to a new location
//    * @param to Route destination
//    * @param options Optional navigation options
//    */
//   abstract push(to: RouteLocationRaw | string, options?: NavigationOptions): Promise<void>
  
//   /**
//    * Navigate to a new location, replacing current history entry
//    * @param to Route destination
//    * @param options Optional navigation options
//    */
//   abstract replace(to: RouteLocationRaw | string, options?: NavigationOptions): Promise<void>
  
//   /**
//    * Go back in history
//    * @param delta Number of steps to go back (default is 1)
//    */
//   abstract back(delta?: number): void
  
//   /**
//    * Go forward in history
//    * @param delta Number of steps to go forward (default is 1)
//    */
//   abstract forward(delta?: number): void
  
//   /**
//    * Get the current route location
//    */
//   abstract getCurrentRoute(): RouteLocationNormalized
  
//   /**
//    * Check if route exists
//    * @param to Route to check
//    */
//   abstract hasRoute(to: string | RouteLocationRaw): boolean
  
//   /**
//    * Generate a URL for a route
//    * @param to Route to resolve
//    */
//   abstract resolve(to: string | RouteLocationRaw): string
  
//   /**
//    * Add a navigation guard that runs before every navigation
//    * @param guard Navigation guard function
//    */
//   abstract beforeEach(guard: NavigationGuard): () => void
  
//   /**
//    * Add a navigation guard that runs after a successful navigation
//    * @param guard Navigation guard function
//    */
//   abstract afterEach(guard: NavigationGuardAfter): () => void
  
//   /**
//    * Add event listener for router events
//    * @param event Event name 
//    * @param handler Event handler
//    */
//   abstract on(event: string, handler: Function): () => void
  
//   /**
//    * Remove event listener
//    * @param event Event name
//    * @param handler Event handler
//    */
//   abstract off(event: string, handler: Function): void
// }

// export interface NavigationOptions {
//   replace?: boolean
//   force?: boolean
//   preserveState?: boolean
// }

// export type NavigationGuardNext = (to?: RouteLocationRaw | false | void) => void

// export interface NavigationGuard {
//   (
//     to: RouteLocationNormalized, 
//     from: RouteLocationNormalized,
//     next: NavigationGuardNext
//   ): void | Promise<void>
// }

// export interface NavigationGuardAfter {
//   (to: RouteLocationNormalized, from: RouteLocationNormalized): void
// }

// export const [injectRouterManager, provideRouterManager] = 
//   createContext<AbstractRouterManager>('root', 'routerManager')