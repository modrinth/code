import { PyroServerError } from "@modrinth/utils";
import { pyroFetch } from "../pyro-fetch.ts";
import { ServerModule } from "./base";
import type {
  JWTAuth,
  DirectoryResponse,
  FilesystemOp,
  FSQueuedOp
} from "@modrinth/utils";

export class FSModule extends ServerModule {
  auth!: JWTAuth;
  ops: FilesystemOp[] = [];
  queuedOps: FSQueuedOp[] = [];
  opsQueuedForModification: string[] = [];

  async fetch(): Promise<void> {
    this.auth = await pyroFetch<JWTAuth>(`servers/${this.serverId}/fs`, {}, "fs");
    this.ops = [];
    this.queuedOps = [];
    this.opsQueuedForModification = [];
  }

  private async retryWithAuth<T>(requestFn: () => Promise<T>): Promise<T> {
    try {
      return await requestFn();
    } catch (error) {
      if (error instanceof PyroServerError && error.statusCode === 401) {
        await this.fetch(); // Refresh auth
        return await requestFn();
      }
      throw error;
    }
  }

  async listDirContents(path: string, page: number, pageSize: number): Promise<DirectoryResponse> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      return await pyroFetch(`/list?path=${encodedPath}&page=${page}&page_size=${pageSize}`, {
        override: this.auth,
        retry: false,
      });
    });
  }

  async createFileOrFolder(path: string, type: "file" | "directory"): Promise<void> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      await pyroFetch(`/create?path=${encodedPath}&type=${type}`, {
        method: "POST",
        contentType: "application/octet-stream",
        override: this.auth,
      });
    });
  }

  async uploadFile(path: string, file: File): Promise<any> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const progressSubject = new EventTarget();
      const abortController = new AbortController();

      const uploadPromise = new Promise((resolve, reject) => {
        const xhr = new XMLHttpRequest();

        xhr.upload.addEventListener("progress", (e) => {
          if (e.lengthComputable) {
            const progress = (e.loaded / e.total) * 100;
            progressSubject.dispatchEvent(
              new CustomEvent("progress", {
                detail: { loaded: e.loaded, total: e.total, progress },
              }),
            );
          }
        });

        xhr.onload = () => {
          if (xhr.status >= 200 && xhr.status < 300) {
            resolve(xhr.response);
          } else {
            reject(new Error(`Upload failed with status ${xhr.status}`));
          }
        };

        xhr.onerror = () => reject(new Error("Upload failed"));
        xhr.onabort = () => reject(new Error("Upload cancelled"));

        xhr.open("POST", `https://${this.auth.url}/create?path=${encodedPath}&type=file`);
        xhr.setRequestHeader("Authorization", `Bearer ${this.auth.token}`);
        xhr.setRequestHeader("Content-Type", "application/octet-stream");
        xhr.send(file);

        abortController.signal.addEventListener("abort", () => xhr.abort());
      });

      return {
        promise: uploadPromise,
        onProgress: (callback: (progress: { loaded: number; total: number; progress: number }) => void) => {
          progressSubject.addEventListener("progress", ((e: CustomEvent) => {
            callback(e.detail);
          }) as EventListener);
        },
        cancel: () => abortController.abort(),
      };
    });
  }

  async renameFileOrFolder(path: string, name: string): Promise<void> {
    const pathName = path.split("/").slice(0, -1).join("/") + "/" + name;
    return this.retryWithAuth(async () => {
      await pyroFetch(`/move`, {
        method: "POST",
        override: this.auth,
        body: { source: path, destination: pathName },
      });
    });
  }

  async updateFile(path: string, content: string): Promise<void> {
    const octetStream = new Blob([content], { type: "application/octet-stream" });
    return this.retryWithAuth(async () => {
      await pyroFetch(`/update?path=${path}`, {
        method: "PUT",
        contentType: "application/octet-stream",
        body: octetStream,
        override: this.auth,
      });
    });
  }

  async moveFileOrFolder(path: string, newPath: string): Promise<void> {
    return this.retryWithAuth(async () => {
      await this.server.createMissingFolders(newPath.substring(0, newPath.lastIndexOf("/")));
      await pyroFetch(`/move`, {
        method: "POST",
        override: this.auth,
        body: { source: path, destination: newPath },
      });
    });
  }

  async deleteFileOrFolder(path: string, recursive: boolean): Promise<void> {
    const encodedPath = encodeURIComponent(path);
    return this.retryWithAuth(async () => {
      await pyroFetch(`/delete?path=${encodedPath}&recursive=${recursive}`, {
        method: "DELETE",
        override: this.auth,
      });
    });
  }

  async downloadFile(path: string, raw?: boolean): Promise<any> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const fileData = await pyroFetch(`/download?path=${encodedPath}`, {
        override: this.auth,
      });

      if (fileData instanceof Blob) {
        return raw ? fileData : await fileData.text();
      }
      return fileData;
    });
  }

  async extractFile(path: string, override = true, dry = false, silentQueue = false): Promise<{ modpack_name: string | null; conflicting_files: string[] }> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);

      if (!silentQueue) {
        this.queuedOps.push({ op: "unarchive", src: path });
        setTimeout(() => this.removeQueuedOp("unarchive", path), 4000);
      }

      try {
        return await pyroFetch(`/unarchive?src=${encodedPath}&trg=/&override=${override}&dry=${dry}`, {
          method: "POST",
          override: this.auth,
          version: 1,
        }, undefined, "Error extracting file");
      } catch (err) {
        this.removeQueuedOp("unarchive", path);
        throw err;
      }
    });
  }

  async modifyOp(id: string, action: "dismiss" | "cancel"): Promise<void> {
    return this.retryWithAuth(async () => {
      await pyroFetch(`/ops/${action}?id=${id}`, {
        method: "POST",
        override: this.auth,
        version: 1,
      }, undefined, `Error ${action === "dismiss" ? "dismissing" : "cancelling"} filesystem operation`);

      this.opsQueuedForModification = this.opsQueuedForModification.filter((x: string) => x !== id);
      this.ops = this.ops.filter((x: FilesystemOp) => x.id !== id);
    });
  }

  removeQueuedOp(op: FSQueuedOp["op"], src: string): void {
    this.queuedOps = this.queuedOps.filter((x: FSQueuedOp) => x.op !== op || x.src !== src);
  }

  clearQueuedOps(): void {
    this.queuedOps = [];
  }
}
