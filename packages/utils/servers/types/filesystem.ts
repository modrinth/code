import type { FSQueuedOp, FilesystemOp } from "./websocket";
import {JWTAuth} from "./api";

export interface DirectoryItem {
  name: string;
  type: "directory" | "file";
  count?: number;
  modified: number;
  created: number;
  path: string;
}

export interface DirectoryResponse {
  items: DirectoryItem[];
  total: number;
  current?: number;
}

export interface FSModule {
  auth: JWTAuth;
  ops: FilesystemOp[];
  queuedOps: FSQueuedOp[];
  opsQueuedForModification: string[];
}
