/**
 * Friends / control-plane types. Live catalog lives in `catalog.ts`.
 * Kept so older imports stay valid.
 */

export type { CatalogPack as FriendPack, CatalogServer } from "./catalog";
export { loadCatalog, installCatalogServer } from "./catalog";
