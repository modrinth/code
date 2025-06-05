import { PyroServerError } from "../errors";

export interface V1ErrorInfo {
  context?: string;
  error: string;
  description: string;
}

export interface JWTAuth {
  url: string;
  token: string;
}

export interface ModuleError {
  error: PyroServerError;
  timestamp: number;
}

export type ModuleName = "general" | "content" | "backups" | "network" | "startup" | "ws" | "fs";
