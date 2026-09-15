import "./styles.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { getLocale, normalizeLocale, setLocale, t } from "./i18n";
import { startParticles } from "./particles";
import { mountPlayingAs } from "./playing-as";
import { createVersionCombobox, type VersionCombobox } from "./version-combobox";
import { loadCatalog, installCatalogServer, type CatalogServer } from "./catalog";
import { owyxLogin, owyxLogout, owyxMe, owyxAdminFetch, owyxApplySkin, type OwyxSession } from "./owyx-auth";
import {
  mountBrowsePage,
  mountProjectPage,
  type BrowseHit,
  type BrowseProjectType,
} from "./browse-page";
import { mountSkinsPage } from "./skins-page";
import { msAuthStart, msAuthPoll, msAuthCancel } from "./ms-auth";

type JavaInstallation = {
  path: string;
  version: string;
  majorVersion: number;
  is64Bit: boolean;
};
type UpdaterConfig = { channel: string; lastCheckAt?: number | null };
type LauncherConfig = {
  nick: string;
  updateBaseUrl?: string | null;
  javaInstallations?: JavaInstallation[];
  javaVersions?: Record<string, string>;
  locale?: string;
  activeProfileId?: string | null;
  updater?: UpdaterConfig;
  defaultMemoryMb?: number;
};
type ProfileKind = "offline" | "microsoft" | "owyx";
type Profile = {
  id: string;
  kind: ProfileKind;
  name: string;
  nick: string;
  needsAuth?: boolean;
  /** Opaque encrypted auth payload — never plaintext tokens. */
  authBlob?: string | null;
  createdAt: number;
};
type ProfilesState = {
  profiles: Profile[];
  activeProfileId?: string | null;
};
type ContentKind = "mod" | "resourcepack" | "shader" | "datapack";
type ContentItem = {
  id: string;
  name: string;
  author: string;
  kind: ContentKind;
  version: string;
  fileName: string;
  enabled: boolean;
};
type WorldItem = {
  id: string;
  name: string;
  mode: string;
  lastPlayedAt: number;
  hardcore?: boolean;
  /** data: URL of the world's icon.png, when present. */
  icon?: string | null;
};
type InstanceSettings = {
  group: string;
  updateChannel: "release" | "beta" | "alpha";
  memoryMb: number;
  javaPath: string;
  width: number;
  height: number;
  jvmArgs: string;
  overrideMemory: boolean;
  overrideJavaPath: boolean;
  overrideJvmArgs: boolean;
  overrideEnvVars: boolean;
  overrideHooks: boolean;
  envVars: string;
  preLaunch: string;
  wrapper: string;
  postExit: string;
};
type Instance = {
  id: string;
  name: string;
  loader: string;
  minecraft: string;
  loaderVersion: string;
  createdAt: number;
  lastPlayedAt?: number;
  installStatus?: "pending" | "installing" | "ready" | "error";
  installMessage?: string;
  /** data: URL when loaded; empty string = none; undefined = not fetched yet */
  iconUrl?: string | null;
  hasIcon?: boolean;
  content: ContentItem[];
  worlds: WorldItem[];
  settings: InstanceSettings;
  catalogServerId?: string;
  catalogPackId?: string;
  serverAddress?: string;
  serverPort?: number;
};

type InstallProgress = {
  packId: string;
  phase: string;
  currentFile?: string | null;
  doneFiles: number;
  totalFiles: number;
  message: string;
  fileBytesDone?: number | null;
  fileBytesTotal?: number | null;
};

const LOADERS = [
  { id: "vanilla", name: "Vanilla" },
  { id: "fabric", name: "Fabric" },
  { id: "forge", name: "Forge" },
  { id: "neoforge", name: "NeoForge" },
  { id: "quilt", name: "Quilt" },
] as const;

const STORAGE_INSTANCES = "owyx.instances";
const STORAGE_PROFILES = "owyx.profiles";
const STORAGE_PROFILES_INIT = "owyx.profilesInitialized";
const STORAGE_NICK = "owyx.nick";
const STORAGE_LOCALE = "owyx.locale";
const STORAGE_ACTIVE_PROFILE = "owyx.activeProfile";

const DEFAULT_SETTINGS = (): InstanceSettings => ({
  group: "",
  updateChannel: "release",
  memoryMb: 4096,
  javaPath: "",
  width: 1280,
  height: 720,
  jvmArgs: "",
  overrideMemory: true,
  overrideJavaPath: false,
  overrideJvmArgs: false,
  overrideEnvVars: false,
  overrideHooks: false,
  envVars: "",
  preLaunch: "",
  wrapper: "",
  postExit: "",
});

type PrefetchStatus = {
  ready: boolean;
  minecraft?: string | null;
  message: string;
  doneFiles: number;
  totalFiles: number;
};

let metaPrefetch: PrefetchStatus = {
  ready: false,
  message: "",
  doneFiles: 0,
  totalFiles: 0,
};
/** Soft-fail: allow Play for already-ready instances if prefetch never completes. */
let metaPrefetchFailed = false;
let systemMemoryMb = 16384;
let javaDetectCallback: ((path: string) => void) | null = null;

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("#app root missing");
const root = app;

const inTauri = Boolean(
  // @ts-expect-error Tauri injects this in WebView
  window.__TAURI_INTERNALS__ ?? window.__TAURI__,
);

metaPrefetch.ready = !inTauri;

let instances: Instance[] = loadInstances();
let selectedInstanceId: string | null = instances[0]?.id ?? null;
let selectedLoader = "fabric";
let mcRequestId = 0;
let loaderRequestId = 0;
let createRefreshEpoch = 0;
let createVersionsReady = false;
let readySpec: {
  loader: string;
  minecraft: string;
  loaderVersion: string;
} | null = null;
let instanceTab: "content" | "files" | "worlds" | "logs" | "share" = "content";
let browseProjectType: BrowseProjectType = "mod";
let browseInstanceId: string | null = null;
let selectedProjectHit: BrowseHit | null = null;
let createWizardStep = 1; // 1 loader → 2 MC → 3 loader ver → 4 name → 5 icon
let owyxSession: OwyxSession | null = null;
let contentFilter: "all" | ContentKind | "enabled" | "disabled" = "all";
let librarySort: "name" | "played" | "created" = "name";
let libraryGroup: "none" | "loader" | "group" = "none";
let libraryFilter: "all" | "modpacks" | "servers" | "custom" = "all";
let instanceSettingsTab:
  | "general"
  | "install"
  | "window"
  | "java"
  | "launch" = "general";
let settingsTab: "java" | "localization" | "updates" | "appearance" | "behavior" = "java";
let activeInstallId: string | null = null;
let profiles: Profile[] = [];
let activeProfileId: string | null = null;
let launcherCfg: LauncherConfig | null = null;
let shellReady = false;
let wizardAllowClose = false;
let profilesPersistChain: Promise<void> = Promise.resolve();

function ensureProfileGate() {
  if (!profiles.length) {
    openProfileWizard({ allowClose: false });
    return;
  }
  if (!activeProfileId || !profiles.some((p) => p.id === activeProfileId)) {
    activeProfileId = profiles[0]?.id ?? null;
  }
}

type NavEntry = { route: string; instanceId?: string | null };
const navHistory: NavEntry[] = [{ route: "home" }];
let navIndex = 0;
let applyingHistory = false;

const HEX_LOGO = `
  <svg viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
    <path d="M16 2L2 9.5V22.5L16 30L30 22.5V9.5L16 2Z" stroke="url(#owyxHex)" stroke-linejoin="round" stroke-width="2"/>
    <path d="M16 8L8 12.5V20L16 24L24 20V12.5L16 8Z" stroke="#00e5ff" stroke-linejoin="round" stroke-width="1.5"/>
    <defs>
      <linearGradient id="owyxHex" x1="16" y1="2" x2="16" y2="30" gradientUnits="userSpaceOnUse">
        <stop stop-color="#8b5cf6"/>
        <stop offset="1" stop-color="#00e5ff"/>
      </linearGradient>
    </defs>
  </svg>`;

root.innerHTML = `
  <canvas id="particles" class="particles" aria-hidden="true"></canvas>
  <div id="splash" class="splash" data-tauri-drag-region>
    <div class="splash-card">
      <div class="splash-logo">${HEX_LOGO}</div>
      <strong class="splash-brand">owyx</strong>
      <div class="splash-bar"><div id="splash-fill" class="splash-fill"></div></div>
      <p id="splash-label" class="splash-label">…</p>
    </div>
    <div class="win-controls splash-win" ${inTauri ? "" : "hidden"}>
            <button type="button" class="win-btn" id="splash-min" data-i18n-title="win.minimize" title="Minimize">─</button>
      <button type="button" class="win-btn" id="splash-max" data-i18n-title="win.maximize" title="Maximize">□</button>
      <button type="button" class="win-btn win-close" id="splash-close" data-i18n-title="win.close" title="Close">✕</button>
    </div>
  </div>
  <div class="shell is-booting" id="shell">
    <aside class="rail" aria-label="Navigation">
      <div class="rail-stack">
        <div class="logo" title="Owyx" aria-label="Owyx">${HEX_LOGO.replaceAll("owyxHex", "owyxHexRail")}</div>
        <nav class="rail-nav">
          <button type="button" class="rail-btn is-active" data-route="home" title="Home">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 10.5 12 4l8 6.5V20a1 1 0 0 1-1 1h-5v-6H10v6H5a1 1 0 0 1-1-1v-9.5Z"/></svg>
          </button>
          <button type="button" class="rail-btn" data-route="browse" title="Discover">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3.5A8.5 8.5 0 1 0 20.5 12 8.5 8.5 0 0 0 12 3.5Zm0 2a6.5 6.5 0 0 1 5.3 10.2l-3.4-3.4a2.5 2.5 0 1 0-1.4 1.4l3.4 3.4A6.5 6.5 0 1 1 12 5.5Z"/></svg>
          </button>
          <button type="button" class="rail-btn" data-route="library" title="Library">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h3v16H5V4Zm5.5 0h3v16h-3V4ZM16 4h3v16h-3V4Z"/></svg>
          </button>
          <button type="button" class="rail-btn" data-route="skins" title="Skins">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 4h6l1 3h3v13H5V7h3l1-3Zm1.2 2 .5-1.5h2.6L13.8 6H10.2ZM8 9v9h8V9H8Zm2 2h4v2h-4v-2Z"/></svg>
          </button>
          <button type="button" class="rail-btn" data-route="servers" title="Private servers">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 5h16v5H4V5Zm0 9h16v5H4v-5Zm3 1.5h2v2H7v-2Zm0-9h2v2H7v-2Z"/></svg>
          </button>
        </nav>
        <div class="rail-sep" role="separator" aria-hidden="true"></div>
        <div id="recent-list" class="recent-list" aria-label="Instances"></div>
        <div class="rail-sep" role="separator" aria-hidden="true"></div>
        <button type="button" class="rail-btn rail-plus" data-action="create" title="New instance">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M11 5h2v6h6v2h-6v6h-2v-6H5v-2h6V5Z"/></svg>
        </button>
        <div class="rail-footer">
        <button type="button" class="rail-btn" data-route="settings" title="Settings">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10.2 3h3.6l.4 2.2a7.2 7.2 0 0 1 1.7.9l2.1-.9 1.8 3.1-1.7 1.4c.1.4.1.8.1 1.2s0 .8-.1 1.2l1.7 1.4-1.8 3.1-2.1-.9a7.2 7.2 0 0 1-1.7.9L13.8 21h-3.6l-.4-2.2a7.2 7.2 0 0 1-1.7-.9l-2.1.9-1.8-3.1 1.7-1.4a7.5 7.5 0 0 1-.1-1.2c0-.4 0-.8.1-1.2L3.2 9.3l1.8-3.1 2.1.9a7.2 7.2 0 0 1 1.7-.9L10.2 3ZM12 9.2A2.8 2.8 0 1 0 12 14.8 2.8 2.8 0 0 0 12 9.2Z"/></svg>
        </button>
        <button type="button" class="rail-btn rail-avatar" data-action="accounts" title="Playing as" id="rail-avatar-btn">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 12a4 4 0 1 0-4-4 4 4 0 0 0 4 4Zm0 2c-4 0-7 2-7 4.5V20h14v-1.5C19 16 16 14 12 14Z"/></svg>
        </button>
        </div>
      </div>
    </aside>

    <div class="stage">
      <header class="topbar" data-tauri-drag-region>
        <div class="topbar-left">
          <strong class="brand">owyx</strong>
          <div class="nav-history">
            <button type="button" class="hist-btn" id="hist-back" title="Back" disabled>‹</button>
            <button type="button" class="hist-btn" id="hist-forward" title="Forward" disabled>›</button>
          </div>
          <span id="crumb" class="crumb"></span>
        </div>
        <div class="topbar-right">
          <div id="download-tip" class="download-tip" hidden aria-hidden="true">
            <span id="download-tip-size" class="download-tip-size"></span>
            <span id="download-tip-speed" class="download-tip-speed"></span>
          </div>
          <div id="top-status" class="pill top-status-pill" title="" role="status" aria-live="polite" data-kind="idle">
            <span id="top-status-text">Ready</span>
            <div class="pill-progress" id="top-progress-track" hidden aria-hidden="true">
              <div id="top-progress-fill" class="pill-progress-fill"></div>
            </div>
          </div>
          <div class="win-controls" ${inTauri ? "" : "hidden"}>
            <button type="button" class="win-btn" id="win-min" data-i18n-title="win.minimize" title="Minimize" aria-label="Minimize">─</button>
            <button type="button" class="win-btn" id="win-max" data-i18n-title="win.maximize" title="Maximize" aria-label="Maximize">□</button>
            <button type="button" class="win-btn win-close" id="win-close" data-i18n-title="win.close" title="Close" aria-label="Close">✕</button>
          </div>
        </div>
      </header>
      <main id="main" class="main"></main>
    </div>
  </div>

  <div id="modal-wizard" class="modal wizard-modal" role="dialog" aria-modal="true" aria-labelledby="wizard-title" hidden>
    <div class="modal-card wizard-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="wizard-title" data-i18n="wizard.title">Create a game profile</h2>
      </header>
      <p id="wizard-sub" class="muted" data-i18n="wizard.subtitle">You need a profile before you can play Minecraft.</p>
      <div id="wizard-kinds" class="wizard-kinds"></div>
      <div class="field" id="wizard-nick-field">
        <label for="wizard-nick" id="wizard-nick-label">Nick</label>
        <input id="wizard-nick" maxlength="16" placeholder="Steve" spellcheck="false" />
      </div>
      <div class="field" id="wizard-auth-field" hidden>
        <button type="button" id="btn-wizard-auth" class="btn btn-ghost" data-i18n="wizard.authBrowser" data-i18n-title="wizard.ms.pipelineTitle">Sign in via browser (soon)</button>
      </div>
      <div id="wizard-owyx-field" hidden>
        <div class="field">
          <label for="wizard-email" id="wizard-email-label">Email</label>
          <input id="wizard-email" type="email" placeholder="you@owyx.site" spellcheck="false" autocomplete="username" />
        </div>
        <div class="field">
          <label for="wizard-password" id="wizard-password-label">Password</label>
          <input id="wizard-password" type="password" placeholder="••••••••" autocomplete="current-password" />
        </div>
      </div>
      <p id="wizard-hint" class="hint"></p>
      <p id="wizard-status" class="status"></p>
      <div class="modal-actions">
        <button type="button" id="btn-wizard" class="btn btn-primary">Create profile</button>
      </div>
    </div>
  </div>

  <div id="modal-create" class="modal" role="dialog" aria-modal="true" aria-labelledby="create-title" hidden>
    <div class="modal-card create-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="create-title" data-i18n="create.title">New instance</h2>
        <button type="button" class="icon-btn" data-close="create" aria-label="Close">✕</button>
      </header>
      <div class="create-steps" id="create-steps" aria-label="Create steps">
        <span class="create-step is-active" data-cstep="1" data-i18n-title="create.step.loader">1</span>
        <span class="create-step-sep"></span>
        <span class="create-step" data-cstep="2" data-i18n-title="create.step.mc">2</span>
        <span class="create-step-sep"></span>
        <span class="create-step" data-cstep="3" data-i18n-title="create.step.loaderVer">3</span>
        <span class="create-step-sep"></span>
        <span class="create-step" data-cstep="4" data-i18n-title="create.step.name">4</span>
        <span class="create-step-sep"></span>
        <span class="create-step" data-cstep="5" data-i18n-title="create.step.icon">5</span>
      </div>
      <p class="muted tiny" id="create-step-label"></p>
      <div class="create-alt-row">
        <button type="button" class="linkish" id="create-type-modpack" data-i18n="create.type.modpack">Browse modpacks</button>
        <button type="button" class="linkish" id="create-import-mrpack" data-i18n="create.type.importMrpack">Import .mrpack</button>
        <span class="muted tiny" data-i18n="create.type.modpackDesc">Discover or import a Modrinth pack (game jars still install on Play).</span>
      </div>
      <div id="create-step-1" class="create-step-panel">
        <div class="field">
          <span class="label" data-i18n="create.loader">Loader</span>
          <div id="loader-chips" class="chips"></div>
        </div>
      </div>
      <div id="create-step-2" class="create-step-panel" hidden>
        <div class="field">
          <span class="label" data-i18n="create.gameVersion">Game version</span>
          <div id="mc-version-host"></div>
        </div>
      </div>
      <div id="create-step-3" class="create-step-panel" hidden>
        <div class="field" id="loader-version-field">
          <span class="label" id="loader-version-label">Fabric version</span>
          <div id="loader-version-host"></div>
        </div>
        <p class="muted tiny" id="create-vanilla-skip" hidden data-i18n="create.vanillaNoLoader">Vanilla has no loader version — continue.</p>
      </div>
      <div id="create-step-4" class="create-step-panel" hidden>
        <div class="field">
          <label for="inst-name" data-i18n="create.name">Name</label>
          <input id="inst-name" maxlength="48" placeholder="" spellcheck="false" />
        </div>
      </div>
      <div id="create-step-5" class="create-step-panel" hidden>
        <div class="create-icon-row">
          <div class="create-icon-preview instance-icon lg" id="create-icon-preview" aria-hidden="true"></div>
          <div class="create-icon-actions">
            <button type="button" class="btn btn-ghost" id="btn-create-icon" data-i18n="create.selectIcon">Select icon</button>
            <button type="button" class="btn btn-ghost" id="btn-create-icon-clear" disabled data-i18n="create.removeIcon">Remove icon</button>
          </div>
        </div>
      </div>
      <p id="create-status" class="status"></p>
      <div class="modal-actions">
        <button type="button" class="btn btn-ghost" id="btn-create-back" data-i18n="create.back">Back</button>
        <button type="button" class="btn btn-primary" id="btn-create-next" data-i18n="create.next">Continue</button>
        <button type="button" id="btn-create" class="btn btn-primary" hidden disabled data-i18n="create.submit">Create</button>
        <button type="button" class="btn btn-ghost" data-close="create" data-i18n="create.cancel">Cancel</button>
      </div>
    </div>
  </div>

  <div id="modal-inst-settings" class="modal" role="dialog" aria-modal="true" aria-labelledby="inst-settings-title" hidden>
    <div class="modal-card settings-modal-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="inst-settings-title">Settings</h2>
        <button type="button" class="icon-btn" data-close="inst-settings" aria-label="Close">✕</button>
      </header>
      <div class="settings-split settings-split-modal">
        <aside class="settings-nav" id="inst-settings-nav"></aside>
        <div class="settings-pane" id="inst-settings-pane"></div>
      </div>
    </div>
  </div>

  <div id="modal-app-settings" class="modal" role="dialog" aria-modal="true" aria-labelledby="app-settings-title" hidden>
    <div class="modal-card settings-modal-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="app-settings-title">Settings</h2>
        <button type="button" class="icon-btn" data-close="app-settings" aria-label="Close">✕</button>
      </header>
      <div class="settings-split settings-split-modal">
        <aside class="settings-nav" id="app-settings-nav"></aside>
        <div class="settings-pane" id="app-settings-pane"></div>
      </div>
    </div>
  </div>

  <div id="modal-java-detect" class="modal" role="dialog" aria-modal="true" aria-labelledby="java-detect-title" hidden>
    <div class="modal-card java-detect-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="java-detect-title">Select Java installation</h2>
        <button type="button" class="icon-btn" data-close="java-detect" aria-label="Close">✕</button>
      </header>
      <div id="java-detect-list" class="java-detect-list"></div>
      <div class="modal-actions">
        <button type="button" class="btn btn-ghost" data-close="java-detect" id="java-detect-cancel">Cancel</button>
      </div>
    </div>
  </div>
`;

const main = root.querySelector<HTMLElement>("#main")!;
const crumb = root.querySelector<HTMLElement>("#crumb")!;
const topStatusEl = root.querySelector<HTMLElement>("#top-status")!;
const topStatusTextEl = root.querySelector<HTMLElement>("#top-status-text")!;
const topProgressTrack = root.querySelector<HTMLElement>("#top-progress-track")!;
const topProgressFill = root.querySelector<HTMLElement>("#top-progress-fill")!;
const downloadTipEl = root.querySelector<HTMLElement>("#download-tip")!;
const downloadTipSizeEl = root.querySelector<HTMLElement>("#download-tip-size")!;
const downloadTipSpeedEl = root.querySelector<HTMLElement>("#download-tip-speed")!;
const topStatus = {
  get textContent() {
    return topStatusTextEl.textContent ?? "";
  },
  set textContent(value: string) {
    const v = value ?? "";
    topStatusTextEl.textContent = v;
    topStatusEl.title = v;
  },
  get dataset() {
    return topStatusEl.dataset;
  },
};

type DownloadTipMeter = {
  fileKey: string;
  lastBytes: number;
  lastAt: number;
  speedMbit: number;
};
let downloadTipMeter: DownloadTipMeter | null = null;

function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${Math.round(n)} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function hideDownloadTip() {
  downloadTipEl.hidden = true;
  downloadTipEl.setAttribute("aria-hidden", "true");
  downloadTipSizeEl.textContent = "";
  downloadTipSpeedEl.textContent = "";
  downloadTipMeter = null;
}

function updateDownloadTip(p: InstallProgress) {
  const done = p.fileBytesDone;
  if (
    done == null ||
    !Number.isFinite(done) ||
    p.phase === "done" ||
    p.phase === "verify" ||
    p.phase === "resolve"
  ) {
    hideDownloadTip();
    return;
  }
  const total =
    p.fileBytesTotal != null && Number.isFinite(p.fileBytesTotal) && p.fileBytesTotal > 0
      ? p.fileBytesTotal
      : null;
  const fileKey = `${p.packId}|${p.currentFile ?? p.message}|${total ?? "?"}`;
  const now = performance.now();
  if (!downloadTipMeter || downloadTipMeter.fileKey !== fileKey) {
    downloadTipMeter = { fileKey, lastBytes: done, lastAt: now, speedMbit: 0 };
  } else {
    const dt = (now - downloadTipMeter.lastAt) / 1000;
    const dBytes = done - downloadTipMeter.lastBytes;
    if (dt >= 0.15 && dBytes >= 0) {
      const instant = (dBytes * 8) / (dt * 1_000_000);
      downloadTipMeter.speedMbit =
        downloadTipMeter.speedMbit > 0
          ? downloadTipMeter.speedMbit * 0.55 + instant * 0.45
          : instant;
      downloadTipMeter.lastBytes = done;
      downloadTipMeter.lastAt = now;
    }
  }
  downloadTipSizeEl.textContent =
    total != null ? `${formatBytes(done)} / ${formatBytes(total)}` : formatBytes(done);
  downloadTipSpeedEl.textContent = `${Math.max(0, downloadTipMeter.speedMbit).toFixed(1)} ${t("download.speedUnit")}`;
  downloadTipEl.hidden = false;
  downloadTipEl.setAttribute("aria-hidden", "false");
}

/** App-wide titlebar: nav/tabs must not overwrite while download/prefetch is busy. */
let titlebarBusy = false;
let titlebarIdleFallback = "";

function showTopProgress(pct: number) {
  const clamped = Math.max(0, Math.min(100, pct));
  topProgressFill.style.width = `${clamped}%`;
  topProgressTrack.hidden = false;
  topStatusEl.classList.add("is-busy");
}
function hideTopProgress() {
  topProgressFill.style.width = "0%";
  topProgressTrack.hidden = true;
  topStatusEl.classList.remove("is-busy");
  hideDownloadTip();
}

function setTitlebarBusy(message: string, pct: number) {
  titlebarBusy = true;
  topStatusEl.dataset.kind = "busy";
  topStatus.textContent = message;
  showTopProgress(pct);
}

function clearTitlebarBusy(idleMessage?: string) {
  titlebarBusy = false;
  hideTopProgress();
  const msg = idleMessage ?? (titlebarIdleFallback || t("status.ready"));
  titlebarIdleFallback = msg;
  topStatusEl.dataset.kind = "idle";
  topStatus.textContent = msg;
}

/** Idle / contextual status — ignored while prefetch or install owns the pill. */
function setTitlebarIdle(message: string) {
  titlebarIdleFallback = message;
  if (titlebarBusy) return;
  topStatusEl.dataset.kind = "idle";
  topStatus.textContent = message;
}

/** Last N status events for the titlebar queue (P1-E / B-005). */
const STATUS_EVENT_LIMIT = 6;
const statusEvents: string[] = [];

function pushStatusEvent(message: string) {
  const msg = message.trim();
  if (!msg) return;
  statusEvents.unshift(msg);
  if (statusEvents.length > STATUS_EVENT_LIMIT) statusEvents.length = STATUS_EVENT_LIMIT;
  const shown =
    statusEvents.length === 1 ? statusEvents[0]! : statusEvents.slice(0, 3).join(" · ");
  setTitlebarIdle(shown);
}

/** https-only Modrinth media (gallery/icons) — mirrors Rust host allowlist. */
function isSafeModrinthMediaUrl(raw: string): boolean {
  try {
    const u = new URL(raw);
    if (u.protocol !== "https:") return false;
    if (u.username || u.password) return false;
    const host = u.hostname.toLowerCase();
    return host === "modrinth.com" || host.endsWith(".modrinth.com");
  } catch {
    return false;
  }
}


function setTitlebarMessage(message: string) {
  topStatusEl.dataset.kind = "info";
  topStatus.textContent = message;
  pushStatusEvent(message);
}

function setTitlebarError(message: string) {
  titlebarBusy = false;
  hideTopProgress();
  topStatusEl.dataset.kind = "error";
  topStatus.textContent = t("status.error", { message });
}
const recentList = root.querySelector<HTMLElement>("#recent-list")!;
const modalCreate = root.querySelector<HTMLElement>("#modal-create")!;
const modalInstSettings = root.querySelector<HTMLElement>("#modal-inst-settings")!;
const modalAppSettings = root.querySelector<HTMLElement>("#modal-app-settings")!;
const modalJavaDetect = root.querySelector<HTMLElement>("#modal-java-detect")!;
const chips = root.querySelector<HTMLElement>("#loader-chips")!;
const mcHost = root.querySelector<HTMLElement>("#mc-version-host")!;
const loaderHost = root.querySelector<HTMLElement>("#loader-version-host")!;
const loaderField = root.querySelector<HTMLElement>("#loader-version-field")!;
const loaderLabel = root.querySelector<HTMLElement>("#loader-version-label")!;
const instName = root.querySelector<HTMLInputElement>("#inst-name")!;
const createStatus = root.querySelector<HTMLElement>("#create-status")!;
const btnCreate = root.querySelector<HTMLButtonElement>("#btn-create")!;
const createIconPreview = root.querySelector<HTMLElement>("#create-icon-preview")!;
const btnCreateIcon = root.querySelector<HTMLButtonElement>("#btn-create-icon")!;
const btnCreateIconClear = root.querySelector<HTMLButtonElement>("#btn-create-icon-clear")!;

let createShowSnapshots = false;
let createIconPath: string | null = null;
let createIconDataUrl: string | null = null;
let runningPollTimer: ReturnType<typeof setInterval> | null = null;

const mcCombo: VersionCombobox = createVersionCombobox({
  searchPlaceholder: "…",
  showAllToggle: true,
  getShowAll: () => createShowSnapshots,
  onShowAllChange: (v) => {
    createShowSnapshots = v;
    void refreshMinecraftVersions();
  },
  onChange: () => {
    refreshCreateNameHint();
    syncCreateWizardStep();
    void refreshLoaderVersions();
  },
});
const loaderCombo: VersionCombobox = createVersionCombobox({
  searchPlaceholder: "…",
  onChange: () => {
    refreshCreateNameHint();
    syncCreateWizardStep();
    if (!createVersionsReady || !readySpec) return;
    if (readySpec.loader !== selectedLoader || readySpec.minecraft !== mcCombo.getValue()) return;
    readySpec = {
      ...readySpec,
      loaderVersion: selectedLoader === "vanilla" ? "" : loaderCombo.getValue(),
    };
  },
});
mcHost.append(mcCombo.root);
loaderHost.append(loaderCombo.root);
const histBack = root.querySelector<HTMLButtonElement>("#hist-back")!;
const histForward = root.querySelector<HTMLButtonElement>("#hist-forward")!;
const shellEl = root.querySelector<HTMLElement>("#shell")!;
const splashEl = root.querySelector<HTMLElement>("#splash")!;
const splashFill = root.querySelector<HTMLElement>("#splash-fill")!;
const splashLabel = root.querySelector<HTMLElement>("#splash-label")!;
const modalWizard = root.querySelector<HTMLElement>("#modal-wizard")!;
const particlesCanvas = root.querySelector<HTMLCanvasElement>("#particles")!;
const modalFocusOrigins = new WeakMap<HTMLElement, HTMLElement>();

function modalFocusable(modal: HTMLElement): HTMLElement[] {
  return Array.from(
    modal.querySelectorAll<HTMLElement>(
      'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ).filter((element) => !element.hidden && element.getClientRects().length > 0);
}

function showModal(modal: HTMLElement, initialSelector?: string) {
  const active = document.activeElement;
  if (active instanceof HTMLElement && !modal.contains(active)) {
    modalFocusOrigins.set(modal, active);
  }
  modal.hidden = false;
  requestAnimationFrame(() => {
    const requested = initialSelector
      ? modal.querySelector<HTMLElement>(initialSelector)
      : null;
    (requested ?? modalFocusable(modal)[0] ?? modal.querySelector<HTMLElement>(".modal-card"))?.focus();
  });
}

function hideModal(modal: HTMLElement) {
  if (modal === modalWizard) {
    stopMsAuthPoll();
  }
  modal.hidden = true;
  const origin = modalFocusOrigins.get(modal);
  modalFocusOrigins.delete(modal);
  origin?.focus();
}

function topVisibleModal(): HTMLElement | null {
  const visible = Array.from(document.querySelectorAll<HTMLElement>(".modal")).filter(
    (modal) => !modal.hidden,
  );
  return visible.at(-1) ?? null;
}

function closeModalFromKeyboard(modal: HTMLElement) {
  if (modal === modalWizard) {
    if (wizardAllowClose) hideModal(modalWizard);
  } else if (modal === modalCreate) {
    closeCreateModal();
  } else if (modal === modalInstSettings) {
    hideModal(modalInstSettings);
  } else if (modal === modalAppSettings) {
    closeAppSettingsOverlay();
  } else if (modal === modalJavaDetect) {
    javaDetectCallback = null;
    hideModal(modalJavaDetect);
  } else if (modal.id === "modal-server-admin") {
    const origin = modalFocusOrigins.get(modal);
    modal.remove();
    origin?.focus();
  }
}

document.addEventListener("keydown", (event) => {
  const modal = topVisibleModal();
  if (!modal) return;
  if (event.key === "Escape") {
    event.preventDefault();
    closeModalFromKeyboard(modal);
    return;
  }
  if (event.key !== "Tab") return;
  const focusable = modalFocusable(modal);
  if (!focusable.length) {
    event.preventDefault();
    modal.querySelector<HTMLElement>(".modal-card")?.focus();
    return;
  }
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
});

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

function activeProfile(): Profile | null {
  return profiles.find((p) => p.id === activeProfileId) ?? profiles[0] ?? null;
}

function profileReady(p: Profile | null): boolean {
  // Offline nick is enough for Play until Microsoft/Owyx auth is implemented.
  return Boolean(p && p.nick.trim());
}

function ensureCanPlay(): boolean {
  const p = activeProfile();
  if (!profileReady(p)) {
    setTitlebarMessage(t("play.noProfile"));
    openProfileWizard({ allowClose: profiles.length > 0 });
    return false;
  }
  if (p?.needsAuth) {
    setTitlebarError(t("play.needsAuth"));
    openProfileWizard({ allowClose: profiles.length > 0 });
    return false;
  }
  return true;
}

function animateMain() {
  main.classList.remove("is-entering");
  // force reflow
  void main.offsetWidth;
  main.classList.add("is-entering");
}

function applyChromeI18n() {
  root.querySelectorAll<HTMLElement>("[data-route]").forEach((btn) => {
    const route = btn.dataset.route!;
    const key =
      route === "home"
        ? "nav.home"
        : route === "library"
          ? "nav.library"
          : route === "browse"
            ? "nav.browse"
            : route === "skins"
              ? "nav.skins"
              : route === "servers"
                ? "nav.servers"
                : route === "stats"
                  ? "nav.stats"
                  : route === "settings"
                    ? "nav.settings"
                    : "";
    if (key) {
      const label = t(key);
      btn.title = label;
      btn.setAttribute("aria-label", label);
    }
  });
  root.querySelectorAll<HTMLElement>("[data-close]").forEach((button) => {
    if (button.textContent?.trim() === "✕") button.setAttribute("aria-label", t("common.close"));
  });
  root.querySelector<HTMLElement>("[data-action='create']")!.title = t("nav.create");
  histBack.title = t("nav.back");
  histForward.title = t("nav.forward");
  if (
    shellReady &&
    topStatusEl.dataset.kind !== "err" &&
    topStatusEl.dataset.kind !== "busy" &&
    (topStatus.textContent === "Ready" ||
      topStatus.textContent === t("status.ready") ||
      topStatusEl.dataset.kind === "idle")
  ) {
    // Refresh idle Ready (and previous-locale Ready) after language switch.
    if (!titlebarBusy) {
      topStatus.textContent = titlebarIdleFallback || t("status.ready");
    }
  }
  root.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    const key = el.dataset.i18n;
    if (key) el.textContent = t(key);
  });
  root.querySelectorAll<HTMLElement>("[data-i18n-title]").forEach((el) => {
    const key = el.dataset.i18nTitle;
    if (!key) return;
    const label = t(key);
    el.title = label;
    if (el.hasAttribute("aria-label")) el.setAttribute("aria-label", label);
  });
}

async function loadProfilesState(): Promise<void> {
  if (inTauri) {
    const state = await invoke<ProfilesState>("get_profiles");
    profiles = state.profiles ?? [];
    activeProfileId = state.activeProfileId ?? null;
    return;
  }
  const initialized = localStorage.getItem(STORAGE_PROFILES_INIT) === "1";
  try {
    profiles = JSON.parse(localStorage.getItem(STORAGE_PROFILES) ?? "[]") as Profile[];
  } catch {
    profiles = [];
  }
  activeProfileId = localStorage.getItem(STORAGE_ACTIVE_PROFILE);
  if (!initialized && !profiles.length) {
    const nick = localStorage.getItem(STORAGE_NICK)?.trim() ?? "";
    if (nick) {
      const id = `prof_${Date.now().toString(36)}`;
      profiles = [
        {
          id,
          kind: "offline",
          name: nick,
          nick,
          needsAuth: false,
          createdAt: Date.now(),
        },
      ];
      activeProfileId = id;
      localStorage.setItem(STORAGE_PROFILES, JSON.stringify(profiles));
      localStorage.setItem(STORAGE_ACTIVE_PROFILE, id);
    }
  }
  localStorage.setItem(STORAGE_PROFILES_INIT, "1");
}

async function persistProfiles(): Promise<void> {
  const run = async () => {
    if (inTauri) {
      const state = await invoke<ProfilesState>("save_profiles", {
        profiles,
        activeProfileId,
      });
      profiles = state.profiles;
      activeProfileId = state.activeProfileId ?? null;
      return;
    }
    localStorage.setItem(STORAGE_PROFILES_INIT, "1");
    localStorage.setItem(STORAGE_PROFILES, JSON.stringify(profiles));
    if (activeProfileId) localStorage.setItem(STORAGE_ACTIVE_PROFILE, activeProfileId);
    else localStorage.removeItem(STORAGE_ACTIVE_PROFILE);
    const p = activeProfile();
    if (p?.kind === "offline") localStorage.setItem(STORAGE_NICK, p.nick);
    else if (!profiles.length) localStorage.removeItem(STORAGE_NICK);
  };
  const prev = profilesPersistChain;
  let release!: () => void;
  profilesPersistChain = new Promise<void>((r) => {
    release = r;
  });
  await prev;
  try {
    await run();
  } finally {
    release();
  }
}

function makeProfile(kind: ProfileKind, nick: string): Profile {
  const clean = nick.trim().slice(0, 16);
  const id = `prof_${Date.now().toString(36)}`;
  const needsAuth = kind !== "offline";
  return {
    id,
    kind,
    name: clean || (kind === "microsoft" ? "Microsoft" : kind === "owyx" ? "Owyx" : "Player"),
    nick: clean || "Player",
    needsAuth,
    // Placeholder for future OS-bound ciphertext (DPAPI / keychain). Never plaintext.
    authBlob: needsAuth ? "pending:encrypted" : null,
    createdAt: Date.now(),
  };
}

let wizardKind: ProfileKind = "offline";

function openProfileWizard(opts?: { allowClose?: boolean }) {
  wizardAllowClose = Boolean(opts?.allowClose);
  wizardKind = "offline";
  stopMsAuthPoll();
  msWizardUiPhase = "idle";
  showModal(modalWizard, "#wizard-nick");
  const title = modalWizard.querySelector("#wizard-title")!;
  const sub = modalWizard.querySelector("#wizard-sub")!;
  const hint = modalWizard.querySelector("#wizard-hint")!;
  const nickLabel = modalWizard.querySelector("#wizard-nick-label")!;
  const btn = modalWizard.querySelector("#btn-wizard")!;
  const nickInput = modalWizard.querySelector<HTMLInputElement>("#wizard-nick")!;
  const nickField = modalWizard.querySelector<HTMLElement>("#wizard-nick-field")!;
  const authField = modalWizard.querySelector<HTMLElement>("#wizard-auth-field")!;
  const authBtn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard-auth")!;
  const kinds = modalWizard.querySelector<HTMLElement>("#wizard-kinds")!;
  const owyxField = modalWizard.querySelector<HTMLElement>("#wizard-owyx-field")!;
  const emailInput = modalWizard.querySelector<HTMLInputElement>("#wizard-email")!;
  const passwordInput = modalWizard.querySelector<HTMLInputElement>("#wizard-password")!;
  const emailLabel = modalWizard.querySelector<HTMLElement>("#wizard-email-label")!;
  const passwordLabel = modalWizard.querySelector<HTMLElement>("#wizard-password-label")!;
  const adding = profiles.length > 0;
  title.textContent = adding ? t("wizard.addTitle") : t("wizard.title");
  sub.textContent = adding ? t("wizard.addSubtitle") : t("wizard.subtitle");
  nickLabel.textContent = t("wizard.nick");
  nickInput.placeholder = t("wizard.nick.placeholder");
    authBtn.textContent = t("wizard.authBrowser");
  authBtn.title = t("wizard.authSoon");
  authBtn.setAttribute("aria-describedby", "wizard-hint");
  emailLabel.textContent = t("wizard.owyx.email");
  passwordLabel.textContent = t("wizard.owyx.password");
  emailInput.placeholder = t("wizard.owyx.email.placeholder");
  hint.textContent = "";
  modalWizard.querySelector<HTMLElement>("#wizard-status")!.textContent = "";

  const primaryLabel = () => {
    if (wizardKind === "owyx") return t("wizard.owyx.signIn");
    return adding ? t("wizard.add") : t("wizard.create");
  };

  const syncWizardKindUi = () => {
    const offline = wizardKind === "offline";
    const owyx = wizardKind === "owyx";
    nickField.hidden = !offline;
    owyxField.hidden = !owyx;
    // Microsoft runs the ms_auth pipeline (path B); Owyx uses the form.
    authField.hidden = offline || owyx;
    const authBtn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard-auth")!;
    authBtn.disabled = false;
    authBtn.removeAttribute("disabled");
    authBtn.textContent = t("wizard.authBrowser");
    authBtn.title = t("wizard.ms.pipelineTitle");
    btn.textContent = primaryLabel();
    if (wizardKind === "microsoft") hint.textContent = t("wizard.security.microsoft");
    else if (owyx) hint.textContent = t("wizard.security.owyx");
    else hint.textContent = "";
  };

  const paintKinds = () => {
    kinds.replaceChildren();
    for (const kind of ["offline", "microsoft", "owyx"] as const) {
      const card = document.createElement("button");
      card.type = "button";
      card.className = `wizard-kind${wizardKind === kind ? " is-selected" : ""}`;
      card.innerHTML = `<strong></strong><span class="muted"></span>`;
      card.querySelector("strong")!.textContent = t(`wizard.kind.${kind}`);
      card.querySelector("span")!.textContent = t(`wizard.kind.${kind}.desc`);
      card.addEventListener("click", () => {
        wizardKind = kind;
        paintKinds();
        syncWizardKindUi();
      });
      kinds.append(card);
    }
    syncWizardKindUi();
  };
  paintKinds();
  nickInput.value = "";
  emailInput.value = "";
  passwordInput.value = "";
  if (wizardKind === "offline") nickInput.focus();
}

/** Map an owyx-auth error code to a friendly, localized message. */
function owyxAuthMessage(codeOrError: unknown): string {
  const code = typeof codeOrError === "string" ? codeOrError : String(codeOrError);
  const key = `wizard.owyx.error.${code}`;
  const msg = t(key);
  // If there is no specific translation, fall back to a generic one.
  return msg === key ? t("wizard.owyx.error.server") : msg;
}

async function finishOwyxWizard() {
  const status = modalWizard.querySelector<HTMLElement>("#wizard-status")!;
  const btn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard")!;
  const emailInput = modalWizard.querySelector<HTMLInputElement>("#wizard-email")!;
  const passwordInput = modalWizard.querySelector<HTMLInputElement>("#wizard-password")!;
  const email = emailInput.value.trim();
  const password = passwordInput.value;

  if (!email || !/^\S+@\S+\.\S+$/.test(email)) {
    setStatus(status, t("wizard.owyx.needEmail"), "err");
    return;
  }
  if (!password || password.length < 6) {
    setStatus(status, t("wizard.owyx.needPassword"), "err");
    return;
  }

  btn.disabled = true;
  setStatus(status, t("wizard.owyx.signingIn"), "muted");
  try {
    const session = await owyxLogin(inTauri, null, email, password);
    if (session.banned) {
      setStatus(status, t("wizard.owyx.error.banned"), "err");
      // Do not create a playable profile for a banned account.
      await owyxLogout(inTauri, session.nickname).catch(() => {});
      return;
    }
    // Real, authenticated Owyx profile: no more needsAuth, Play is unlocked.
    const profile: Profile = {
      id: `prof_${Date.now().toString(36)}`,
      kind: "owyx",
      name: session.nickname || "Owyx",
      nick: session.nickname || "Player",
      needsAuth: false,
      authBlob: "owyx:session",
      createdAt: Date.now(),
    };
    profiles = [profile, ...profiles.filter((p) => p.id !== profile.id)];
    activeProfileId = profile.id;
    owyxSession = session;
    await persistProfiles();
    void applyAccountSkin(session.nickname);
    hideModal(modalWizard);
    if (shellReady) {
      applyingHistory = true;
      const route = currentNav().route === "instance" ? "home" : currentNav().route || "home";
      routeTo(route);
      applyingHistory = false;
    }
  } catch (e) {
    setStatus(status, owyxAuthMessage(e), "err");
  } finally {
    btn.disabled = false;
  }
}

type MsWizardUiPhase = "idle" | "starting" | "waiting" | "scaffold" | "error" | "unconfigured";

let msAuthPollTimer: ReturnType<typeof setInterval> | null = null;
let msAuthPollInFlight = false;
let msWizardUiPhase: MsWizardUiPhase = "idle";

function stopMsAuthPoll() {
  if (msAuthPollTimer) {
    clearInterval(msAuthPollTimer);
    msAuthPollTimer = null;
  }
  msAuthPollInFlight = false;
  void msAuthCancel();
  if (msWizardUiPhase === "waiting" || msWizardUiPhase === "starting") {
    msWizardUiPhase = "idle";
  }
  const authBtn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard-auth");
  if (authBtn && !modalWizard.hidden) {
    authBtn.disabled = false;
    if (msWizardUiPhase === "idle") {
      authBtn.textContent = t("wizard.authBrowser");
    }
  }
}

function paintMsAuthButton() {
  const authBtn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard-auth");
  if (!authBtn) return;
  authBtn.disabled = false;
  if (msWizardUiPhase === "waiting" || msWizardUiPhase === "starting") {
    authBtn.textContent = t("wizard.ms.cancel");
  } else if (msWizardUiPhase === "scaffold" || msWizardUiPhase === "unconfigured" || msWizardUiPhase === "error") {
    authBtn.textContent = t("wizard.ms.retry");
  } else {
    authBtn.textContent = t("wizard.authBrowser");
  }
}

/** Single click source for #btn-wizard-auth (no onclick mixing). */
async function onMicrosoftAuthButtonClick() {
  if (msWizardUiPhase === "waiting" || msWizardUiPhase === "starting") {
    stopMsAuthPoll();
    msWizardUiPhase = "idle";
    const status = modalWizard.querySelector<HTMLElement>("#wizard-status")!;
    setStatus(status, t("wizard.ms.cancelled"), "muted");
    paintMsAuthButton();
    return;
  }
  await startMicrosoftAuthFromWizard();
}

async function startMicrosoftAuthFromWizard() {
  const status = modalWizard.querySelector<HTMLElement>("#wizard-status")!;
  const hint = modalWizard.querySelector<HTMLElement>("#wizard-hint")!;
  const authBtn = modalWizard.querySelector<HTMLButtonElement>("#btn-wizard-auth")!;
  stopMsAuthPoll();
  msWizardUiPhase = "starting";
  authBtn.disabled = true;
  authBtn.textContent = t("wizard.ms.starting");
  setStatus(status, t("wizard.ms.starting"), "muted");
  hint.textContent = t("wizard.security.microsoft");

  const started = await msAuthStart();
  if (started.phase === "unconfigured") {
    msWizardUiPhase = "unconfigured";
    setStatus(status, t("wizard.ms.unconfigured"), "err");
    hint.textContent = t("wizard.ms.unconfiguredHint");
    paintMsAuthButton();
    return;
  }
  if (started.phase === "scaffold" || started.phase === "not_implemented") {
    msWizardUiPhase = "scaffold";
    setStatus(status, t("wizard.ms.scaffold"), "muted");
    hint.textContent = started.message || t("wizard.ms.scaffoldHint");
    paintMsAuthButton();
    return;
  }
  if (started.phase === "error") {
    msWizardUiPhase = "error";
    setStatus(status, started.message || t("wizard.ms.error"), "err");
    paintMsAuthButton();
    return;
  }
  if (started.phase !== "waiting") {
    msWizardUiPhase = "error";
    setStatus(status, started.message || t("wizard.ms.error"), "err");
    paintMsAuthButton();
    return;
  }

  // Real device-code waiting (only when Rust returns waiting + codes).
  msWizardUiPhase = "waiting";
  if (started.userCode && started.verificationUri) {
    hint.textContent = t("wizard.ms.codeHint", {
      code: started.userCode,
      uri: started.verificationUri,
    });
  } else {
    hint.textContent = started.message || t("wizard.ms.waiting");
  }
  setStatus(status, t("wizard.ms.waiting"), "muted");
  paintMsAuthButton();

  msAuthPollTimer = setInterval(() => {
    if (msAuthPollInFlight) return;
    msAuthPollInFlight = true;
    void (async () => {
      try {
        const st = await msAuthPoll();
        if (st.phase === "success") {
          stopMsAuthPoll();
          msWizardUiPhase = "idle";
          const profile = makeProfile("microsoft", st.gamertag || "Player");
          if (st.authBlob && st.authBlob.startsWith("ms1:")) {
            profile.needsAuth = false;
            profile.authBlob = st.authBlob;
            profiles = [profile, ...profiles.filter((p) => p.id !== profile.id)];
            activeProfileId = profile.id;
            await persistProfiles();
            setStatus(status, t("wizard.ms.success"), "ok");
            hint.textContent = t("wizard.ms.successHint");
            paintMsAuthButton();
            hideModal(modalWizard);
            if (shellReady) {
              applyingHistory = true;
              const route = currentNav().route === "instance" ? "home" : currentNav().route || "home";
              routeTo(route);
              applyingHistory = false;
            }
            return;
          }
          // No real blob — keep needsAuth (should not happen on Path A).
          profile.needsAuth = true;
          profile.authBlob = "pending:encrypted";
          profiles = [profile, ...profiles.filter((p) => p.id !== profile.id)];
          activeProfileId = profile.id;
          await persistProfiles();
          setStatus(status, t("wizard.ms.successPending"), "muted");
          hint.textContent = t("wizard.ms.successPendingHint");
          paintMsAuthButton();
          return;
        }
        if (st.phase === "scaffold" || st.phase === "not_implemented") {
          stopMsAuthPoll();
          msWizardUiPhase = "scaffold";
          setStatus(status, t("wizard.ms.scaffold"), "muted");
          hint.textContent = st.message || t("wizard.ms.scaffoldHint");
          paintMsAuthButton();
          return;
        }
        if (st.phase === "error" || st.phase === "unconfigured") {
          stopMsAuthPoll();
          msWizardUiPhase = st.phase === "unconfigured" ? "unconfigured" : "error";
          setStatus(status, st.message || t("wizard.ms.error"), "err");
          paintMsAuthButton();
        }
      } finally {
        msAuthPollInFlight = false;
      }
    })();
  }, 2000);
}

async function finishWizard() {
  const nickInput = modalWizard.querySelector<HTMLInputElement>("#wizard-nick")!;
  const status = modalWizard.querySelector<HTMLElement>("#wizard-status")!;
  const nick = nickInput.value.trim();

  // Owyx account: real sign-in against the site API.
  if (wizardKind === "owyx") {
    await finishOwyxWizard();
    return;
  }

  if (wizardKind === "offline") {
    if (!nick || !/^[A-Za-z0-9_]+$/.test(nick)) {
      setStatus(status, t("wizard.needNick"), "err");
      return;
    }
  }
  const profile = makeProfile(wizardKind, wizardKind === "offline" ? nick : nick || "Player");
  profiles = [profile, ...profiles.filter((p) => p.id !== profile.id)];
  activeProfileId = profile.id;
  try {
    stopMsAuthPoll();
    await persistProfiles();
    hideModal(modalWizard);
    if (shellReady) {
      applyingHistory = true;
      const route = currentNav().route === "instance" ? "home" : currentNav().route || "home";
      routeTo(route);
      applyingHistory = false;
    }
  } catch (e) {
    setStatus(status, String(e), "err");
  }
}

function setSplash(progress: number, key: string) {
  splashFill.style.width = `${Math.max(0, Math.min(100, progress))}%`;
  splashLabel.textContent = t(key);
}

async function boot() {
  setLocale(normalizeLocale(localStorage.getItem(STORAGE_LOCALE)));
  setSplash(8, "splash.boot");
  await sleep(180);
  setSplash(28, "splash.config");
  try {
    if (inTauri) {
      launcherCfg = await invoke<LauncherConfig>("get_config");
      setLocale(normalizeLocale(launcherCfg.locale));
    } else {
      setLocale(normalizeLocale(localStorage.getItem(STORAGE_LOCALE)));
    }
  } catch {
    /* keep default locale */
  }
  await sleep(160);
  if (inTauri) {
    try {
      systemMemoryMb = await invoke<number>("system_memory_mb");
    } catch {
      systemMemoryMb = 16384;
    }
    // Kick off shared meta prefetch during splash (Modrinth-like), not only on Play.
    try {
      metaPrefetch = await invoke<PrefetchStatus>("meta_prefetch_status");
      if (!metaPrefetch.ready) {
        void invoke<PrefetchStatus>("prefetch_shared_meta")
          .then((s) => {
            metaPrefetch = s;
            if (s.ready) {
              metaPrefetchFailed = false;
              if (!activeInstallId) clearTitlebarBusy(t("status.ready"));
            }
          })
          .catch((e) => {
            metaPrefetchFailed = true;
            if (!activeInstallId) {
              titlebarBusy = false;
              hideTopProgress();
              setTitlebarMessage(
                `${t("meta.failedContinue")} (${String(e).slice(0, 80)})`,
              );
            }
          });
      }
    } catch {
      /* ignore */
    }
  }

  setSplash(55, "splash.profiles");
  await loadProfilesState();
  await refreshOwyxSession();
  await sleep(140);
  setSplash(78, "splash.i18n");
  applyChromeI18n();
  await sleep(140);
  setSplash(100, "splash.ready");
  await sleep(220);

  splashEl.classList.add("is-done");
  shellEl.classList.remove("is-booting");
  shellReady = true;
  startParticles(particlesCanvas);
  await hydrateInstancesFromDisk();

  if (inTauri && !metaPrefetch.ready && !metaPrefetchFailed) {
    setTitlebarBusy(
      t("meta.prefetching"),
      metaPrefetch.totalFiles > 0
        ? (metaPrefetch.doneFiles / metaPrefetch.totalFiles) * 100
        : 5,
    );
  } else {
    setTitlebarIdle(t("status.ready"));
  }

  updateHistoryButtons();
  renderRecent();
  applyingHistory = true;
  routeTo("home");
  applyingHistory = false;
  updateHistoryButtons();
  ensureProfileGate();
}

function currentNav(): NavEntry {
  return navHistory[navIndex] ?? { route: "home" };
}

function sameNav(a: NavEntry, b: NavEntry) {
  return a.route === b.route && (a.instanceId ?? null) === (b.instanceId ?? null);
}

function updateHistoryButtons() {
  histBack.disabled = navIndex <= 0;
  histForward.disabled = navIndex >= navHistory.length - 1;
}

function pushHistory(entry: NavEntry) {
  if (sameNav(currentNav(), entry)) {
    updateHistoryButtons();
    return;
  }
  navHistory.splice(navIndex + 1);
  navHistory.push(entry);
  navIndex = navHistory.length - 1;
  updateHistoryButtons();
}

function applyNav(entry: NavEntry) {
  applyingHistory = true;
  routeTo(entry.route, entry.instanceId ?? undefined);
  applyingHistory = false;
  updateHistoryButtons();
}

function goBack() {
  if (navIndex <= 0) return;
  navIndex -= 1;
  applyNav(currentNav());
}

function goForward() {
  if (navIndex >= navHistory.length - 1) return;
  navIndex += 1;
  applyNav(currentNav());
}

function normalizeInstance(raw: Partial<Instance> & Pick<Instance, "id" | "name" | "loader" | "minecraft">): Instance {
  return {
    id: raw.id,
    name: raw.name,
    loader: raw.loader,
    minecraft: raw.minecraft,
    loaderVersion: raw.loaderVersion ?? "",
    createdAt: raw.createdAt ?? Date.now(),
    lastPlayedAt: raw.lastPlayedAt,
    content: Array.isArray(raw.content) ? raw.content : [],
    worlds: Array.isArray(raw.worlds) ? raw.worlds : [],
    settings: { ...DEFAULT_SETTINGS(), ...(raw.settings ?? {}) },
    installStatus: raw.installStatus,
    installMessage: raw.installMessage,
    hasIcon: raw.hasIcon,
    iconUrl: raw.iconUrl,
    catalogServerId: raw.catalogServerId,
    catalogPackId: raw.catalogPackId,
    serverAddress: raw.serverAddress,
    serverPort: raw.serverPort,
  };
}

function loadInstances(): Instance[] {
  try {
    const raw = localStorage.getItem(STORAGE_INSTANCES);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Partial<Instance>[];
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((i) => i && typeof i.id === "string" && typeof i.name === "string")
      .map((i) => normalizeInstance(i as Instance));
  } catch {
    return [];
  }
}

function saveInstances() {
  const slim = instances.map(({ iconUrl: _iconUrl, ...rest }) => rest);
  localStorage.setItem(STORAGE_INSTANCES, JSON.stringify(slim));
}

type DiskInstance = {
  id: string;
  name: string;
  loader: string;
  minecraft: string;
  loaderVersion: string;
  createdAt: number;
  status: string;
  ready: boolean;
  path: string;
  hasIcon?: boolean;
  catalogServerId?: string;
  catalogPackId?: string;
  serverAddress?: string;
  serverPort?: number;
};

function createdAtMs(value: number): number {
  if (!value) return Date.now();
  return value < 1e12 ? value * 1000 : value;
}

function statusFromDisk(d: DiskInstance): Instance["installStatus"] {
  if (d.status === "installing") {
    // Stale "installing" after crash/restart must not trap UI — only live install stays installing.
    return activeInstallId === d.id ? "installing" : "pending";
  }
  if (d.status === "error") return "error";
  if (d.ready || d.status === "ready") return "ready";
  return "pending";
}

async function hydrateInstancesFromDisk(): Promise<void> {
  if (!inTauri) return;
  try {
    const disk = await invoke<DiskInstance[]>("list_instances");
    const prevById = new Map(instances.map((i) => [i.id, i]));
    const merged = disk.map((d) => {
      const prev = prevById.get(d.id);
      return normalizeInstance({
        id: d.id,
        name: d.name || prev?.name || d.id,
        loader: d.loader || prev?.loader || "vanilla",
        minecraft: d.minecraft || prev?.minecraft || "",
        loaderVersion: d.loaderVersion || prev?.loaderVersion || "",
        createdAt: createdAtMs(d.createdAt) || prev?.createdAt || Date.now(),
        lastPlayedAt: prev?.lastPlayedAt,
        content: prev?.content ?? [],
        worlds: prev?.worlds ?? [],
        settings: prev?.settings ?? DEFAULT_SETTINGS(),
        installStatus: statusFromDisk(d),
        installMessage:
          prev?.installMessage ||
          (d.ready ? t("instance.installDone") : d.status === "error" ? t("instance.installFail") : undefined),
        hasIcon: d.hasIcon ?? prev?.hasIcon,
        iconUrl: d.hasIcon ? prev?.iconUrl : null,
        catalogServerId: d.catalogServerId,
        catalogPackId: d.catalogPackId,
        serverAddress: d.serverAddress,
        serverPort: d.serverPort,
      });
    });
    const diskIds = new Set(disk.map((d) => d.id));
    const localOnly = instances.filter(
      (i) =>
        !diskIds.has(i.id) &&
        (i.installStatus === "pending" || i.installStatus === "installing"),
    );
    instances = [...merged, ...localOnly];
    if (selectedInstanceId && !instances.some((i) => i.id === selectedInstanceId)) {
      selectedInstanceId = instances[0]?.id ?? null;
    }
    saveInstances();
  } catch (e) {
    console.warn("hydrateInstancesFromDisk", e);
  }
}

async function deleteInstanceEverywhere(inst: Instance): Promise<void> {
  if (inTauri) {
    try {
      await invoke("delete_instance", { id: inst.id });
    } catch (e) {
      topStatus.textContent = friendlyError(e);
      return;
    }
  }
  instances = instances.filter((i) => i.id !== inst.id);
  saveInstances();
  selectedInstanceId = instances[0]?.id ?? null;
  routeTo("library");
}

function prettyLoader(id: string) {
  return LOADERS.find((l) => l.id === id)?.name ?? id;
}

function applyInstanceIcon(el: Element, inst: Instance) {
  const node = el as HTMLElement;
  node.classList.toggle("has-image", Boolean(inst.iconUrl));
  node.replaceChildren();
  if (inst.iconUrl) {
    const img = document.createElement("img");
    img.src = inst.iconUrl;
    img.alt = "";
    node.append(img);
  } else {
    node.textContent = inst.name.slice(0, 1).toUpperCase();
  }
  if (inst.hasIcon && inst.iconUrl == null && inTauri) {
    void invoke<string | null>("get_instance_icon", { id: inst.id })
      .then((url) => {
        inst.iconUrl = url;
        inst.hasIcon = Boolean(url);
        if (node.isConnected) applyInstanceIcon(node, inst);
      })
      .catch(() => {
        inst.iconUrl = null;
        inst.hasIcon = false;
      });
  }
}

function stopRunningPoll() {
  if (runningPollTimer) {
    clearInterval(runningPollTimer);
    runningPollTimer = null;
  }
}

function startRunningPoll(inst: Instance) {
  stopRunningPoll();
  if (!inTauri) return;
  runningPollTimer = setInterval(() => {
    void (async () => {
      if (selectedInstanceId !== inst.id) {
        stopRunningPoll();
        return;
      }
      try {
        const running = await invoke<boolean>("is_pack_running", { packId: inst.id });
        if (!running) {
          stopRunningPoll();
          setTitlebarIdle(t("play.exited"));
          await refreshPlayButton(inst);
        }
      } catch {
        /* ignore */
      }
    })();
  }, 1200);
}

function setStatus(el: HTMLElement, text: string, kind: "ok" | "err" | "muted" = "muted") {
  el.textContent = text;
  el.dataset.kind = kind;
}

function beginCreateRefresh() {
  createRefreshEpoch += 1;
  createVersionsReady = false;
  readySpec = null;
  btnCreate.disabled = true;
  syncCreateWizardStep();
  return createRefreshEpoch;
}

function setCreateReady(epoch: number, ready: boolean) {
  if (epoch !== createRefreshEpoch) return;
  createVersionsReady = ready;
  btnCreate.disabled = !ready;
  if (ready) {
    readySpec = {
      loader: selectedLoader,
      minecraft: mcCombo.getValue(),
      loaderVersion: selectedLoader === "vanilla" ? "" : loaderCombo.getValue(),
    };
  } else {
    readySpec = null;
  }
  syncCreateWizardStep();
}

function isProfileGateOpen(): boolean {
  return !modalWizard.hidden && !wizardAllowClose;
}

/** Map content route → rail highlight (Modrinth-like active states). */
function railRouteFor(route: string): string {
  if (route === "instance") return "library";
  if (route === "project") return browseInstanceId ? "library" : "browse";
  if (route === "browse") return "browse";
  if (route === "skins") return "skins";
  if (route === "stats") return "home";
  return route;
}

function setRailHighlight(route: string) {
  const railRoute = railRouteFor(route);
  root.querySelectorAll<HTMLButtonElement>(".rail-btn[data-route]").forEach((btn) => {
    const active = btn.dataset.route === railRoute;
    btn.classList.toggle("is-active", active);
    if (active) btn.setAttribute("aria-current", "page");
    else btn.removeAttribute("aria-current");
  });
}

function contentRailRoute(): string {
  return railRouteFor(currentNav().route);
}

/**
 * Settings is a modal overlay — not a content-route in history (P0-A / P1-F).
 * Rail highlights Settings while open; close restores the underlying content route.
 */
function openAppSettingsOverlay() {
  if (isProfileGateOpen()) return;
  setRailHighlight("settings");
  void openAppSettingsModal();
}

function closeAppSettingsOverlay() {
  hideModal(modalAppSettings);
  setRailHighlight(contentRailRoute());
}

function routeTo(route: string, instanceId?: string) {
  if (isProfileGateOpen()) return;
  // Settings stays an overlay; never push it into content history.
  if (route === "settings") {
    openAppSettingsOverlay();
    return;
  }
  if (!modalAppSettings.hidden) hideModal(modalAppSettings);
  if (route === "browse") {
    browseInstanceId = instanceId ?? null;
    if (instanceId) selectedInstanceId = instanceId;
  } else if (route === "project") {
    if (instanceId) {
      browseInstanceId = instanceId;
      selectedInstanceId = instanceId;
    }
  } else if (route !== "project" && route !== "instance") {
    // leave browseInstanceId when returning via history
  }
  const entry: NavEntry = {
    route,
    instanceId:
      route === "instance" || route === "browse" || route === "project"
        ? (instanceId ?? selectedInstanceId ?? browseInstanceId)
        : null,
  };
  if (!applyingHistory) pushHistory(entry);

  setRailHighlight(route);
  if (instanceId && (route === "instance" || route === "browse" || route === "project")) {
    selectedInstanceId = instanceId;
  }
  paintCurrentRoute(route, instanceId);
}

/** Re-render the visible content route without pushing history (locale hot-reload). */
function paintCurrentRoute(route?: string, instanceId?: string) {
  const nav = currentNav();
  // Skip leftover "settings" history entries — paint underlying content instead.
  let r = route ?? nav.route;
  let id = instanceId ?? nav.instanceId ?? undefined;
  if (r === "settings") {
    for (let i = navIndex - 1; i >= 0; i--) {
      const prev = navHistory[i];
      if (prev && prev.route !== "settings") {
        r = prev.route;
        id = prev.instanceId ?? undefined;
        break;
      }
    }
    if (r === "settings") r = "home";
  }
  animateMain();
  if (r === "home") renderHome();
  else if (r === "library") {
    void hydrateInstancesFromDisk().finally(() => renderLibrary());
  } else if (r === "servers") renderServers();
  else if (r === "stats") renderStats();
  else if (r === "skins") renderSkins();
  else if (r === "browse") {
    if (id) browseInstanceId = id;
    void hydrateInstancesFromDisk().finally(() => renderBrowse());
  } else if (r === "project") {
    if (id) browseInstanceId = id;
    void hydrateInstancesFromDisk().finally(() => renderProject());
  } else if (r === "instance") {
    if (id) selectedInstanceId = id;
    void hydrateInstancesFromDisk().finally(() => renderInstance());
  }
  renderRecent();
  updateHistoryButtons();
}

/** Modrinth-like library kind: origin/link, not loader. */
function instanceLibraryKind(inst: Instance): "server" | "modpack" | "custom" {
  if (inst.catalogServerId) return "server";
  if (inst.catalogPackId) return "modpack";
  return "custom";
}

function renderRecent() {
  const recent = [...instances]
    .sort((a, b) => (b.lastPlayedAt ?? b.createdAt) - (a.lastPlayedAt ?? a.createdAt))
    .slice(0, 6);
  recentList.replaceChildren();
  for (const inst of recent) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `recent-btn${inst.id === selectedInstanceId ? " is-active" : ""}`;
    btn.title = `${inst.name} · ${prettyLoader(inst.loader)} ${inst.minecraft}`;
    const hue = Math.abs(hashHue(inst.id)) % 360;
    btn.style.setProperty("--recent-hue", String(hue));
    btn.textContent = inst.name.slice(0, 1).toUpperCase();
    btn.addEventListener("click", () => {
      selectedInstanceId = inst.id;
      instanceTab = "content";
      routeTo("instance", inst.id);
    });
    recentList.append(btn);
  }
}

function hashHue(s: string) {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return h;
}

function formatRelative(ts?: number) {
  if (!ts) return t("time.never");
  const diff = Date.now() - ts;
  const min = Math.floor(diff / 60000);
  if (min < 1) return t("time.now");
  if (min < 60) return t("time.minutesAgo", { n: min });
  const hrs = Math.floor(min / 60);
  if (hrs < 48) return t("time.hoursAgo", { n: hrs });
  const days = Math.floor(hrs / 24);
  return t("time.daysAgo", { n: days });
}

function renderHome() {
  crumb.textContent = ` / ${t("nav.home")}`;
  setTitlebarIdle(t("status.ready"));
  const recent = [...instances]
    .sort((a, b) => (b.lastPlayedAt ?? b.createdAt) - (a.lastPlayedAt ?? a.createdAt))
    .slice(0, 8);
  main.innerHTML = `
    <section class="page home-page">
      <div class="home-top">
        <div class="home-main">
          <header class="page-head">
            <h1>${recent.length ? t("home.welcomeBack") : t("home.welcomeNew")}</h1>
            <p class="muted">${recent.length ? t("home.subtitleBack") : t("home.subtitleNew")}</p>
          </header>
          <section class="home-section">
            <h2 class="section-title">${t("home.jump")}</h2>
            <div id="jump-list" class="jump-list"></div>
            <div class="home-quick-actions home-quick-always" id="home-quick-row">
              <button type="button" class="btn btn-ghost sm" id="home-qa-create-2">${t("home.quick.create")}</button>
              <button type="button" class="btn btn-ghost sm" id="home-qa-library-2">${t("home.quick.library")}</button>
              <button type="button" class="btn btn-ghost sm" id="home-qa-servers-2">${t("home.quick.servers")}</button>
            </div>
          </section>
        </div>
        <aside class="home-aside" id="home-profile-slot"></aside>
      </div>
      <section class="home-section news-section news-full">
        <h2 class="section-title">${t("home.news")}</h2>
        <div class="news-grid">
          <article class="news-tile">
            <span class="news-tag">${t("home.news.launcher")}</span>
            <h3>${t("home.news.brand.title")}</h3>
            <p class="muted">${t("home.news.brand.body")}</p>
          </article>
          <article class="news-tile">
            <span class="news-tag">${t("home.news.servers")}</span>
            <h3>${t("home.news.servers.title")}</h3>
            <p class="muted">${t("home.news.servers.body")}</p>
          </article>
          <article class="news-tile">
            <span class="news-tag">${t("home.news.launcher")}</span>
            <h3>${t("home.news.profiles.title")}</h3>
            <p class="muted">${t("home.news.profiles.body")}</p>
          </article>
          <article class="news-tile">
            <span class="news-tag">${t("home.news.launcher")}</span>
            <h3>${t("home.news.update.title")}</h3>
            <p class="muted">${t("home.news.update.body")}</p>
          </article>
        </div>
      </section>
    </section>
  `;

  mountPlayingAs(main.querySelector("#home-profile-slot")!, {
    getProfiles: () => profiles,
    getActiveId: () => activeProfileId,
    setActiveId: (id) => {
      activeProfileId = id;
    },
    setProfiles: (next) => {
      profiles = next;
    },
    persist: async () => {
      await persistProfiles();
      await refreshOwyxSession();
    },
    openWizard: openProfileWizard,
    setStatus,
    onRemove: (p) => {
      // Forget any stored Owyx session when its profile is deleted (logout).
      if (p.kind === "owyx") void owyxLogout(inTauri, p.nick);
    },
  });

  if (!profiles.length) {
    window.setTimeout(() => ensureProfileGate(), 0);
  }

  const jumpList = main.querySelector<HTMLElement>("#jump-list")!;
  main.querySelector("#home-qa-create-2")?.addEventListener("click", () => {
    openCreateModal();
  });
  main.querySelector("#home-qa-library-2")?.addEventListener("click", () => routeTo("library"));
  main.querySelector("#home-qa-servers-2")?.addEventListener("click", () => routeTo("servers"));
  if (!recent.length) {
    jumpList.innerHTML = `
      <div class="empty-state jump-empty">
        <h3>${t("home.jumpEmpty")}</h3>
        <p class="muted">${t("home.jumpEmptyHint")}</p>
        <div class="home-quick-actions">
          <button type="button" class="btn btn-primary" id="home-qa-create">${t("home.quick.create")}</button>
          <button type="button" class="btn btn-ghost" id="home-qa-library">${t("home.quick.library")}</button>
          <button type="button" class="btn btn-ghost" id="home-qa-servers">${t("home.quick.servers")}</button>
        </div>
      </div>`;
    jumpList.querySelector("#home-qa-create")?.addEventListener("click", () => {
      openCreateModal();
    });
    jumpList.querySelector("#home-qa-library")?.addEventListener("click", () => routeTo("library"));
    jumpList.querySelector("#home-qa-servers")?.addEventListener("click", () => routeTo("servers"));
    main.querySelector("#home-quick-row")?.classList.add("is-hidden");
    return;
  }
  main.querySelector("#home-quick-row")?.classList.remove("is-hidden");
  for (const inst of recent) jumpList.append(jumpBackRow(inst));
}

function jumpBackRow(inst: Instance) {
  const row = document.createElement("div");
  row.className = "jump-row";
  row.innerHTML = `
    <button type="button" class="jump-open">
      <span class="instance-icon"></span>
      <span class="instance-meta">
        <strong></strong>
        <small></small>
      </span>
    </button>
    <div class="jump-actions">
      <button type="button" class="btn btn-primary btn-play">▶ ${t("library.play")}</button>
      <button type="button" class="icon-btn sm" data-open title="${t("common.open")}">↗</button>
    </div>
  `;
  applyInstanceIcon(row.querySelector(".instance-icon")!, inst);
  row.querySelector("strong")!.textContent = inst.name;
  row.querySelector("small")!.textContent =
    `${formatRelative(inst.lastPlayedAt)} · ${prettyLoader(inst.loader)} ${inst.minecraft}`;

  const open = () => {
    selectedInstanceId = inst.id;
    instanceTab = "content";
    routeTo("instance", inst.id);
  };
  row.querySelector(".jump-open")?.addEventListener("click", open);
  row.querySelector("[data-open]")?.addEventListener("click", open);
  row.querySelector(".btn-play")?.addEventListener("click", () => {
    if (!ensureCanPlay()) return;
    inst.lastPlayedAt = Date.now();
    saveInstances();
    renderRecent();
    void tryLaunch(inst);
    const jumpList = main.querySelector<HTMLElement>("#jump-list");
    if (jumpList && main.querySelector(".home-page")) {
      jumpList.replaceChildren();
      const recent = [...instances]
        .sort((a, b) => (b.lastPlayedAt ?? b.createdAt) - (a.lastPlayedAt ?? a.createdAt))
        .slice(0, 8);
      for (const item of recent) jumpList.append(jumpBackRow(item));
    }
  });
  return row;
}

function instanceRow(inst: Instance) {
  const row = document.createElement("div");
  row.className = "instance-row";
  row.innerHTML = `
    <button type="button" class="instance-open">
      <span class="instance-icon"></span>
      <span class="instance-meta">
        <strong></strong>
        <small></small>
      </span>
    </button>
    <div class="instance-actions">
      <button type="button" class="btn btn-primary btn-play">▶ ${t("library.play")}</button>
    </div>
  `;
  applyInstanceIcon(row.querySelector(".instance-icon")!, inst);
  row.querySelector("strong")!.textContent = inst.name;
  row.querySelector("small")!.textContent =
    `${prettyLoader(inst.loader)} ${inst.minecraft}` +
    (inst.loaderVersion ? ` · ${inst.loaderVersion}` : "") +
    ` · ${formatRelative(inst.lastPlayedAt)}`;

  const open = () => {
    selectedInstanceId = inst.id;
    instanceTab = "content";
    routeTo("instance", inst.id);
  };
  row.querySelector(".instance-open")?.addEventListener("click", open);
  row.querySelector(".btn-play")?.addEventListener("click", (e) => {
    e.stopPropagation();
    if (!ensureCanPlay()) return;
    inst.lastPlayedAt = Date.now();
    saveInstances();
    renderRecent();
    void tryLaunch(inst);
  });
  return row;
}

function renderLibrary() {
  crumb.textContent = ` / ${t("library.title")}`;
  setTitlebarIdle(t("status.ready"));
  main.innerHTML = `
    <section class="page">
      <header class="page-head row">
        <div>
          <h1>${t("library.title")}</h1>
          <div class="tabs" id="lib-tabs">
            <button type="button" class="tab${libraryFilter === "all" ? " is-active" : ""}" data-lib-filter="all">${t("library.all")}</button>
            <button type="button" class="tab${libraryFilter === "modpacks" ? " is-active" : ""}" data-lib-filter="modpacks">${t("library.modpacks")}</button>
            <button type="button" class="tab${libraryFilter === "servers" ? " is-active" : ""}" data-lib-filter="servers">${t("library.servers")}</button>
            <button type="button" class="tab${libraryFilter === "custom" ? " is-active" : ""}" data-lib-filter="custom">${t("library.custom")}</button>
          </div>
        </div>
        <button type="button" class="btn btn-primary" data-action="create">${t("library.new")}</button>
      </header>
      <div class="toolbar lib-toolbar">
        <input class="search" placeholder="${t("library.search")}" id="lib-search" />
        <div class="select-wrap slim">
          <select id="lib-sort">
            <option value="name">${t("library.sort.name")}</option>
            <option value="played">${t("library.sort.played")}</option>
            <option value="created">${t("library.sort.created")}</option>
          </select>
        </div>
        <div class="select-wrap slim">
          <select id="lib-group">
            <option value="none">${t("library.group.none")}</option>
            <option value="group">${t("library.group.group")}</option>
            <option value="loader">${t("library.group.loader")}</option>
          </select>
        </div>
      </div>
      <div id="lib-list" class="instance-list"></div>
    </section>
  `;
  const list = main.querySelector<HTMLElement>("#lib-list")!;
  const search = main.querySelector<HTMLInputElement>("#lib-search")!;
  const sortSel = main.querySelector<HTMLSelectElement>("#lib-sort")!;
  const groupSel = main.querySelector<HTMLSelectElement>("#lib-group")!;
  sortSel.value = librarySort;
  groupSel.value = libraryGroup;

  const matchesFilter = (inst: Instance) => {
    const kind = instanceLibraryKind(inst);
    if (libraryFilter === "servers") return kind === "server";
    if (libraryFilter === "modpacks") return kind === "modpack";
    if (libraryFilter === "custom") return kind === "custom";
    return true;
  };

  const paint = () => {
    list.replaceChildren();
    librarySort = sortSel.value as typeof librarySort;
    libraryGroup = groupSel.value as typeof libraryGroup;
    const q = search.value.trim().toLowerCase();
    let filtered = instances.filter(
      (i) => matchesFilter(i) && (!q || i.name.toLowerCase().includes(q)),
    );
    filtered = [...filtered].sort((a, b) => {
      if (librarySort === "played") {
        return (b.lastPlayedAt ?? 0) - (a.lastPlayedAt ?? 0);
      }
      if (librarySort === "created") return b.createdAt - a.createdAt;
      return a.name.localeCompare(b.name, "en");
    });
    if (!filtered.length) {
      const emptyTitle =
        libraryFilter === "servers"
          ? t("library.empty.servers")
          : libraryFilter === "modpacks"
            ? t("library.empty.modpacks")
            : libraryFilter === "custom"
              ? t("library.empty.custom")
              : t("library.empty");
      const emptyHint =
        libraryFilter === "all" ? t("library.emptyHint") : t("library.emptyHint.filter");
      list.innerHTML = `
        <div class="empty-state no-instance">
          <h3>${emptyTitle}</h3>
          <p class="muted">${emptyHint}</p>
          <button type="button" class="btn btn-primary" data-action="create">${t("library.new")}</button>
        </div>`;
      return;
    }

    const appendGroup = (title: string | null, items: Instance[]) => {
      if (title) {
        const h = document.createElement("h3");
        h.className = "group-title";
        h.textContent = title;
        list.append(h);
      }
      for (const inst of items) list.append(instanceRow(inst));
    };

    if (libraryGroup === "none") {
      appendGroup(null, filtered);
      return;
    }
    const map = new Map<string, Instance[]>();
    for (const inst of filtered) {
      const key =
        libraryGroup === "loader"
          ? prettyLoader(inst.loader)
          : inst.settings.group.trim() || t("library.group.ungrouped");
      const bucket = map.get(key) ?? [];
      bucket.push(inst);
      map.set(key, bucket);
    }
    for (const [title, items] of [...map.entries()].sort((a, b) => a[0].localeCompare(b[0], "en"))) {
      appendGroup(title, items);
    }
  };
  search.addEventListener("input", paint);
  sortSel.addEventListener("change", paint);
  groupSel.addEventListener("change", paint);
  main.querySelectorAll<HTMLButtonElement>("[data-lib-filter]").forEach((btn) => {
    btn.addEventListener("click", () => {
      libraryFilter = (btn.dataset.libFilter ?? "all") as typeof libraryFilter;
      main.querySelectorAll<HTMLButtonElement>("[data-lib-filter]").forEach((b) => {
        b.classList.toggle("is-active", b === btn);
      });
      paint();
    });
  });
  paint();
}

function isOwyxAdmin(): boolean {
  return owyxSession?.role === "admin";
}

async function refreshOwyxSession(): Promise<void> {
  const p = activeProfile();
  if (!p || p.kind !== "owyx" || !p.nick) {
    owyxSession = null;
    return;
  }
  try {
    owyxSession = await owyxMe(inTauri, p.nick);
    void applyAccountSkin(p.nick);
  } catch (e) {
    owyxSession = null;
    if (String(e) === "no_session") {
      p.needsAuth = true;
      await persistProfiles().catch(() => {});
    }
  }
  if (currentNav().route === "servers") renderServers();
}

async function applyAccountSkin(nickname: string): Promise<void> {
  try {
    const res = await owyxApplySkin(inTauri, nickname, selectedInstanceId);
    if (res.applied) {
      setTitlebarIdle(t("skin.applied"));
    }
  } catch {
    /* skin is optional */
  }
}

function renderServers() {
  crumb.textContent = ` / ${t("nav.servers")}`;
  setTitlebarIdle(t("status.ready"));
  main.innerHTML = `
    <section class="page">
      <header class="page-head row">
        <div>
          <h1>${t("servers.title")}</h1>
          <p class="muted">${t("servers.body")}</p>
        </div>
        <div class="page-head-actions">
          <button type="button" class="btn btn-ghost" id="btn-servers-refresh">${t("common.refresh")}</button>
          ${isOwyxAdmin() ? `<button type="button" class="btn btn-primary" id="btn-servers-add">${t("servers.admin.add")}</button>` : ""}
        </div>
      </header>
      <div id="servers-list" class="server-grid"><p class="muted">${t("servers.loading")}</p></div>
    </section>
  `;
  main.querySelector("#btn-servers-refresh")?.addEventListener("click", () => void paintServersList());
  main.querySelector("#btn-servers-add")?.addEventListener("click", () => openServerAdminModal());
  void paintServersList();
}

async function paintServersList() {
  const box = main.querySelector<HTMLElement>("#servers-list");
  if (!box) return;
  box.innerHTML = `<p class="muted">${t("servers.loading")}</p>`;
  const catalog = await loadCatalog(inTauri);
  if (catalog.error && !catalog.servers.length) {
    box.innerHTML = `
      <div class="empty-state servers-empty">
        <div class="servers-empty-art" aria-hidden="true"></div>
        <h3>${t("servers.offline.title")}</h3>
        <p class="muted">${t("servers.offline.body")}</p>
        <div class="home-quick-actions">
          <button type="button" class="btn btn-ghost" id="btn-servers-retry">${t("common.refresh")}</button>
          <button type="button" class="btn btn-ghost" id="btn-servers-to-library-off">${t("home.quick.library")}</button>
        </div>
      </div>`;
    box.querySelector("#btn-servers-retry")?.addEventListener("click", () => void paintServersList());
    box.querySelector("#btn-servers-to-library-off")?.addEventListener("click", () => routeTo("library"));
    return;
  }
  if (!catalog.servers.length) {
    box.innerHTML = `
      <div class="empty-state servers-empty">
        <div class="servers-empty-art" aria-hidden="true"></div>
        <h3>${t("servers.empty.title")}</h3>
        <p class="muted">${t("servers.empty.body")}</p>
        <div class="home-quick-actions">
          <button type="button" class="btn btn-ghost" id="btn-servers-retry-empty">${t("common.refresh")}</button>
          <button type="button" class="btn btn-ghost" id="btn-servers-to-library">${t("home.quick.library")}</button>
        </div>
      </div>`;
    box.querySelector("#btn-servers-retry-empty")?.addEventListener("click", () => void paintServersList());
    box.querySelector("#btn-servers-to-library")?.addEventListener("click", () => routeTo("library"));
    return;
  }
  box.replaceChildren();
  for (const server of catalog.servers) {
    const inst = instances.find((i) => i.catalogServerId === server.id);
    const card = document.createElement("article");
    card.className = "server-card";
    const address = `${server.address}:${server.port}`;
    const version = [server.minecraft || server.pack?.minecraft, server.loader || server.pack?.loader]
      .filter(Boolean)
      .join(" · ");
    const status = server.status?.online;
    const statusLabel =
      status === true
        ? server.status?.players != null && server.status?.max != null
          ? `${t("servers.online")} · ${t("servers.players", { players: server.status.players, max: server.status.max })}`
          : t("servers.online")
        : status === false
          ? t("servers.offline")
          : t("servers.statusUnknown");
    const unavailable =
      server.pack != null &&
      (server.pack.downloadAvailable === false ||
        server.pack.sourceType === "sftp");
    const unavailableReason = t("servers.packUnavailable");
    card.innerHTML = `
      <div class="server-card-icon" aria-hidden="true"></div>
      <div class="server-card-body">
        <h3></h3>
        <p class="muted tiny"></p>
        <p class="muted tiny server-addr"></p>
        <div class="server-card-meta">
          <span class="server-status" data-online="${String(status)}"></span>
          ${server.requiresAccount ? `<span class="server-account">${t("servers.accountRequired")}</span>` : ""}
        </div>
      </div>
      <div class="server-card-actions">
        <button type="button" class="btn btn-ghost sm" data-act="download"></button>
        <button type="button" class="btn btn-primary sm" data-act="play"></button>
      </div>`;
    card.querySelector("h3")!.textContent = server.name;
    card.querySelector(".tiny")!.textContent = version || t("servers.unknownVersion");
    card.querySelector(".server-addr")!.textContent = address;
    card.querySelector(".server-status")!.textContent = statusLabel;
    if (server.iconUrl) {
      try {
        const iconUrl = new URL(server.iconUrl);
        if (iconUrl.protocol === "https:" || iconUrl.protocol === "http:") {
          const img = document.createElement("img");
          img.src = iconUrl.toString();
          img.alt = "";
          img.loading = "lazy";
          card.querySelector(".server-card-icon")!.append(img);
        }
      } catch {
        // Keep the branded fallback when an admin supplied a bad icon URL.
      }
    }
    const dl = card.querySelector<HTMLButtonElement>("[data-act=download]")!;
    const play = card.querySelector<HTMLButtonElement>("[data-act=play]")!;
    dl.textContent = inst?.installStatus === "ready" ? t("servers.update") : t("servers.download");
    play.textContent = t("servers.play");
    dl.disabled = unavailable;
    play.disabled = unavailable;
    if (unavailable) {
      dl.title = unavailableReason;
      play.title = unavailableReason;
    }
    dl.addEventListener("click", () => {
      if (dl.disabled) return;
      dl.disabled = true;
      void downloadCatalogServer(server).finally(() => {
        const current = instances.find((i) => i.catalogServerId === server.id);
        dl.disabled = unavailable;
        dl.textContent = current?.installStatus === "ready" ? t("servers.update") : t("servers.download");
      });
    });
    play.addEventListener("click", () => {
      if (play.disabled) return;
      void playCatalogServer(server);
    });
    box.append(card);
  }
}

async function downloadCatalogServer(server: CatalogServer): Promise<Instance | null> {
  if (
    server.pack &&
    (server.pack.downloadAvailable === false || server.pack.sourceType === "sftp")
  ) {
    setTitlebarError(t("servers.packUnavailable"));
    return null;
  }
  if (
    server.requiresAccount &&
    (activeProfile()?.kind !== "owyx" ||
      activeProfile()?.needsAuth ||
      owyxSession?.serverAccess !== true)
  ) {
    setTitlebarError(t("servers.needAccount"));
    openProfileWizard({ allowClose: true });
    return null;
  }
  if (!inTauri) {
    setTitlebarMessage(t("play.browser"));
    const id = `srv-${server.id}`;
    const inst = upsertCatalogInstance(server, id, "pending");
    return inst;
  }
  setTitlebarBusy(t("servers.downloading"), 8);
  try {
    const res = await installCatalogServer(inTauri, server);
    const inst = upsertCatalogInstance(server, res.instanceId, res.ready ? "ready" : "error");
    inst.installMessage = res.message;
    saveInstances();
    await hydrateInstancesFromDisk();
    const found = instances.find((i) => i.id === res.instanceId) ?? inst;
    found.catalogServerId = server.id;
    found.catalogPackId = server.packId ?? undefined;
    found.serverAddress = server.address;
    found.serverPort = server.port;
    saveInstances();
    clearTitlebarBusy(t("servers.downloaded"));
    return found;
  } catch (e) {
    setTitlebarError(friendlyError(e));
    return null;
  }
}

function upsertCatalogInstance(server: CatalogServer, id: string, status: Instance["installStatus"]): Instance {
  const prev = instances.find((i) => i.id === id || i.catalogServerId === server.id);
  const inst = normalizeInstance({
    id,
    name: server.name,
    loader: server.loader || server.pack?.loader || "vanilla",
    minecraft: server.minecraft || server.pack?.minecraft || "1.21.1",
    loaderVersion: prev?.loaderVersion || "",
    createdAt: prev?.createdAt ?? Date.now(),
    settings: prev?.settings ?? DEFAULT_SETTINGS(),
    content: prev?.content ?? [],
    worlds: prev?.worlds ?? [],
    installStatus: status,
    catalogServerId: server.id,
    catalogPackId: server.packId ?? undefined,
    serverAddress: server.address,
    serverPort: server.port,
  });
  instances = [inst, ...instances.filter((i) => i.id !== inst.id && i.catalogServerId !== server.id)];
  saveInstances();
  return inst;
}

async function playCatalogServer(server: CatalogServer) {
  if (
    server.requiresAccount &&
    (activeProfile()?.kind !== "owyx" ||
      activeProfile()?.needsAuth ||
      owyxSession?.serverAccess !== true)
  ) {
    setTitlebarError(t("servers.needAccount"));
    openProfileWizard({ allowClose: true });
    return;
  }
  let inst = instances.find((i) => i.catalogServerId === server.id) ?? null;
  if (!inst || inst.installStatus !== "ready") {
    inst = await downloadCatalogServer(server);
  }
  if (!inst) return;
  inst.serverAddress = server.address;
  inst.serverPort = server.port;
  saveInstances();
  await tryLaunch(inst);
}

function openServerAdminModal() {
  const existing = document.querySelector("#modal-server-admin");
  existing?.remove();
  const modal = document.createElement("div");
  modal.id = "modal-server-admin";
  modal.className = "modal";
  modal.setAttribute("role", "dialog");
  modal.setAttribute("aria-modal", "true");
  modal.setAttribute("aria-labelledby", "server-admin-title");
  modal.innerHTML = `
    <div class="modal-card" tabindex="-1">
      <header class="modal-head">
        <h2 id="server-admin-title">${t("servers.admin.add")}</h2>
        <button type="button" class="icon-btn" data-close aria-label="${t("common.close")}">×</button>
      </header>
      <form id="server-admin-form" class="form-grid">
        <label>${t("servers.admin.name")}<input class="input" name="name" required /></label>
        <label>${t("servers.admin.address")}<input class="input" name="address" required placeholder="play.owyx.site" /></label>
        <label>${t("servers.admin.port")}<input class="input" name="port" type="number" value="25565" /></label>
        <label>${t("servers.admin.minecraft")}<input class="input" name="minecraft" value="1.21.1" /></label>
        <label>${t("servers.admin.loader")}
          <select class="select" name="loader">
            <option value="vanilla">vanilla</option>
            <option value="fabric">fabric</option>
            <option value="forge">forge</option>
            <option value="neoforge">neoforge</option>
            <option value="quilt">quilt</option>
          </select>
        </label>
        <label>${t("servers.admin.packUrl")}<input class="input" name="packUrl" placeholder="https://…/pack.zip" /></label>
        <p class="muted tiny">${t("servers.admin.packCap")}</p>
        <p class="muted tiny" id="server-admin-status"></p>
        <div class="modal-actions">
          <button type="button" class="btn btn-ghost" data-close>${t("common.cancel")}</button>
          <button type="submit" class="btn btn-primary">${t("common.save")}</button>
        </div>
      </form>
    </div>`;
  document.body.append(modal);
  showModal(modal, 'input[name="name"]');
  modal.querySelectorAll("[data-close]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const origin = modalFocusOrigins.get(modal);
      modal.remove();
      origin?.focus();
    });
  });
  modal.querySelector("#server-admin-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    void submitServerAdmin(modal);
  });
}

async function submitServerAdmin(modal: HTMLElement) {
  const form = modal.querySelector<HTMLFormElement>("#server-admin-form")!;
  const status = modal.querySelector<HTMLElement>("#server-admin-status")!;
  const nick = activeProfile()?.nick;
  if (!nick || !owyxSession || owyxSession.role !== "admin") {
    status.textContent = t("servers.admin.needAdmin");
    return;
  }
  const fd = new FormData(form);
  const name = String(fd.get("name") || "").trim();
  const address = String(fd.get("address") || "").trim();
  const port = parseInt(String(fd.get("port") || "25565"), 10);
  const minecraft = String(fd.get("minecraft") || "1.21.1").trim();
  const loader = String(fd.get("loader") || "vanilla").trim();
  const packUrl = String(fd.get("packUrl") || "").trim();
  status.textContent = t("servers.admin.saving");
  try {
    let packId: string | undefined;
    if (packUrl) {
      const pack = await owyxAdminFetch(inTauri, nick, "POST", "/api/admin/packs", {
        name: `${name} pack`,
        minecraft,
        loader,
        sourceType: "http_zip",
        source: { type: "http_zip", config: { url: packUrl } },
        published: true,
      });
      packId = String((pack.pack as { id?: string } | undefined)?.id || "");
    }
    await owyxAdminFetch(inTauri, nick, "POST", "/api/admin/servers", {
      name,
      address,
      port,
      kind: "owyx",
      packId: packId || null,
      minecraft,
      loader,
      requiresAccount: false,
      published: true,
    });
    modal.remove();
    setTitlebarIdle(t("servers.admin.created"));
    void paintServersList();
  } catch (e) {
    status.textContent = owyxAuthMessage(e);
  }
}

function renderStats() {
  crumb.textContent = ` / ${t("nav.stats")}`;
  setTitlebarIdle(t("status.ready"));
  main.innerHTML = `
    <section class="page coming-soon">
      <div class="coming-soon-card">
        <div class="coming-soon-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="40" height="40" fill="currentColor"><path d="M5 19h3v-6H5v6Zm5.5 0h3V5h-3v14ZM16 19h3v-9h-3v9Z"/></svg>
        </div>
        <h1>${t("stats.title")}</h1>
        <p class="muted">${t("stats.body")}</p>
        <span class="pill static-pill">${t("stats.soon")}</span>
      </div>
    </section>
  `;
}

function renderInstance() {
  const inst = instances.find((i) => i.id === selectedInstanceId);
  if (!inst) {
    routeTo("library");
    return;
  }
  crumb.textContent = ` / ${inst.name}`;
  setTitlebarIdle(`${prettyLoader(inst.loader)} ${inst.minecraft}`);
  main.innerHTML = `
    <section class="page instance-page">
      <header class="instance-hero">
        <div class="instance-hero-left">
          <div class="instance-icon lg"></div>
          <div>
            <h1></h1>
            <p class="muted" id="inst-sub"></p>
          </div>
        </div>
        <div class="instance-hero-actions">
          <button type="button" class="btn btn-primary btn-play-main" id="btn-play">▶ ${t("instance.play")}</button>
          <button type="button" class="icon-btn icon-btn-square" id="btn-inst-settings" title="${t("instance.settings")}" aria-label="${t("instance.settings")}">⚙</button>
          <div class="menu-wrap">
            <button type="button" class="icon-btn icon-btn-square" id="btn-inst-more" title="${t("instance.more")}" aria-label="${t("instance.more")}">⋮</button>
            <div id="inst-more-menu" class="menu" hidden>
              <button type="button" data-more="open-folder">${t("instance.openFolder")}</button>
              <button type="button" data-more="dup">${t("instance.dup")}</button>
              <button type="button" data-more="settings">${t("instance.settings")}</button>
              <button type="button" data-more="delete" class="danger">${t("instance.delete")}</button>
            </div>
          </div>
        </div>
      </header>
      <div class="tabs" id="inst-tabs">
        <button type="button" class="tab" data-tab="content">${t("instance.content")}</button>
        <button type="button" class="tab" data-tab="files">${t("instance.files")}</button>
        <button type="button" class="tab" data-tab="worlds">${t("instance.worlds")}</button>
        <button type="button" class="tab" data-tab="logs">${t("instance.logs")}</button>
        <button type="button" class="tab" data-tab="share">${t("instance.share")}</button>
      </div>
      <div id="inst-panel" class="card content-card"></div>
    </section>
  `;
  applyInstanceIcon(main.querySelector(".instance-icon")!, inst);
  main.querySelector("h1")!.textContent = inst.name;
  main.querySelector("#inst-sub")!.textContent =
    `Minecraft ${inst.minecraft} · ${prettyLoader(inst.loader)}` +
    (inst.loaderVersion ? ` ${inst.loaderVersion}` : "") +
    (inst.installStatus ? ` · ${t(`instance.status.${inst.installStatus}`)}` : "");

  if (inst.installMessage) {
    setTitlebarIdle(inst.installMessage);
  }

  const tabs = main.querySelectorAll<HTMLButtonElement>("#inst-tabs .tab");
  tabs.forEach((tabBtn) => {
    tabBtn.classList.toggle("is-active", tabBtn.dataset.tab === instanceTab);
    tabBtn.addEventListener("click", () => {
      instanceTab = tabBtn.dataset.tab as typeof instanceTab;
      renderInstance();
    });
  });

  const panel = main.querySelector<HTMLElement>("#inst-panel")!;
  if (instanceTab === "content") renderContentPanel(panel, inst);
  else if (instanceTab === "files") renderFilesPanel(panel, inst);
  else if (instanceTab === "worlds") renderWorldsPanel(panel, inst);
  else if (instanceTab === "share") renderSharePanel(panel, inst);
  else renderLogsPanel(panel, inst);

  main.querySelector("#btn-play")?.addEventListener("click", () => {
    void onPlayClick(inst);
  });
  main.querySelector("#btn-inst-settings")?.addEventListener("click", () => {
    instanceSettingsTab = "general";
    renderInstanceSettings(inst);
  });
  void refreshPlayButton(inst);

  const moreBtn = main.querySelector<HTMLButtonElement>("#btn-inst-more")!;
  const moreMenu = main.querySelector<HTMLElement>("#inst-more-menu")!;
  moreBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    moreMenu.hidden = !moreMenu.hidden;
  });
  moreMenu.addEventListener("click", (e) => e.stopPropagation());
  moreMenu.querySelectorAll<HTMLButtonElement>("[data-more]").forEach((btn) => {
    btn.addEventListener("click", () => {
      moreMenu.hidden = true;
      const action = btn.dataset.more;
      if (action === "open-folder") {
        if (!inTauri) {
          setTitlebarMessage(t("files.openBrowser"));
          return;
        }
        void invoke("open_instance_dir", { id: inst.id, which: "root" }).catch((e) => {
          setTitlebarError(friendlyError(e));
        });
      } else if (action === "settings") {
        instanceSettingsTab = "general";
        renderInstanceSettings(inst);
      } else if (action === "dup") {
        void duplicateInstanceOnDisk(inst);
      } else if (action === "delete") {
        void deleteInstanceEverywhere(inst);
      }
    });
  });
}

window.addEventListener("click", () => {
  document.querySelectorAll<HTMLElement>(".menu").forEach((m) => {
    m.hidden = true;
  });
});

function kindLabel(kind: ContentKind) {
  if (kind === "mod") return t("content.kind.mod");
  if (kind === "resourcepack") return t("content.kind.resourcepack");
  if (kind === "datapack") return t("content.kind.datapack");
  return t("content.kind.shader");
}

function browseDepsBase() {
  return {
    t,
    inTauri,
    isSafeModrinthMediaUrl,
    friendlyError,
    prettyLoader,
    setStatus: (msg: string) => {
      setTitlebarMessage(msg);
    },
  };
}

function renderBrowse() {
  const inst = browseInstanceId
    ? instances.find((i) => i.id === browseInstanceId) ?? null
    : null;
  if (inst) {
    crumb.textContent = ` / ${inst.name} › ${t("browse.contentInstall")}`;
    setTitlebarIdle(
      t("browse.installMeta", {
        mc: inst.minecraft,
        loader: prettyLoader(inst.loader),
      }),
    );
  } else {
    crumb.textContent = ` / ${t("nav.browse")}`;
    setTitlebarIdle(t("status.ready"));
  }
  main.replaceChildren();
  mountBrowsePage(main, {
    deps: {
      ...browseDepsBase(),
      onOpenProject: (hit, projectType) => {
        selectedProjectHit = hit;
        browseProjectType = projectType;
        routeTo("project", inst?.id ?? browseInstanceId ?? undefined);
      },
      onBackToContent: () => {
        if (inst) {
          selectedInstanceId = inst.id;
          instanceTab = "content";
          routeTo("instance", inst.id);
        } else {
          routeTo("home");
        }
      },
      getInstalledProjectIds: async () => {
        const ids = new Set<string>();
        if (!inst || !inTauri) return ids;
        try {
          const rows = await invoke<
            Array<{ id: string; name: string; modrinthProjectId?: string | null }>
          >("list_content", { id: inst.id });
          for (const r of rows) {
            const pid = r.modrinthProjectId?.trim();
            if (pid) ids.add(pid);
          }
        } catch {
          /* ignore */
        }
        return ids;
      },
      onModpackImported: async (instanceId) => {
        await hydrateInstancesFromDisk();
        selectedInstanceId = instanceId;
        routeTo("instance", instanceId);
      },
    },
    instance: inst
      ? { id: inst.id, name: inst.name, loader: inst.loader, minecraft: inst.minecraft }
      : null,
    projectType: browseProjectType,
    onProjectTypeChange: (pt) => {
      browseProjectType = pt;
    },
  });
}

function renderProject() {
  if (!selectedProjectHit) {
    routeTo("browse", browseInstanceId ?? undefined);
    return;
  }
  const inst = browseInstanceId
    ? instances.find((i) => i.id === browseInstanceId) ?? null
    : null;
  const hit = selectedProjectHit;
  crumb.textContent = inst
    ? ` / ${inst.name} › ${hit.title}`
    : ` / ${t("nav.browse")} › ${hit.title}`;
  setTitlebarIdle(hit.title);
  main.replaceChildren();
  mountProjectPage(main, hit, {
    ...browseDepsBase(),
    instance: inst
      ? { id: inst.id, name: inst.name, loader: inst.loader, minecraft: inst.minecraft }
      : null,
    projectType: browseProjectType,
    onOpenProject: () => {},
    onBack: () => routeTo("browse", inst?.id ?? browseInstanceId ?? undefined),
    onModpackImported: async (instanceId) => {
      await hydrateInstancesFromDisk();
      selectedInstanceId = instanceId;
      routeTo("instance", instanceId);
    },
  });
}

function renderSkins() {
  crumb.textContent = ` / ${t("nav.skins")}`;
  setTitlebarIdle(t("status.ready"));
  const prof = activeProfile();
  main.replaceChildren();
  mountSkinsPage(main, {
    t,
    inTauri,
    activeNick: prof?.nick ?? null,
    owyxSkinUrl: owyxSession?.skinUrl ?? null,
    setStatus: (msg, kind) => {
      if (kind === "err") setTitlebarError(msg);
      else setTitlebarMessage(msg);
    },
    listSkins: async () => invoke("list_skins"),
    applySkin: async (id) => {
      await invoke("apply_local_skin", {
        skinId: id,
        instanceId: selectedInstanceId,
      });
    },
    removeSkin: async (id) => {
      await invoke("remove_skin", { skinId: id });
    },
    onApplyOwyxSkin: async () => {
      const nick = prof?.nick;
      if (!nick) throw new Error(t("skins.needOwyx"));
      await applyAccountSkin(nick);
    },
    onImportFile: async () => {
      if (!inTauri) {
        setTitlebarMessage(t("skins.importBrowser"));
        return;
      }
      try {
        const selected = await openDialog({
          multiple: false,
          title: t("skins.import"),
          filters: [{ name: "PNG", extensions: ["png"] }],
        });
        if (typeof selected !== "string") return;
        await invoke("import_skin", { path: selected, name: null });
        setTitlebarMessage(t("skins.imported"));
      } catch (e) {
        setTitlebarError(friendlyError(e));
      }
    },
  });
}

function renderContentPanel(panel: HTMLElement, inst: Instance) {
  panel.innerHTML = `
    <div class="toolbar content-toolbar">
      <input class="search" id="content-search" placeholder="${t("content.search")}" />
      <button type="button" class="btn btn-ghost" id="btn-add-content">${t("content.uploadFiles")}</button>
      <button type="button" class="btn btn-primary" id="btn-browse-content">${t("content.browseContent")}</button>
      <button type="button" class="btn btn-ghost" data-open-mods>${t("content.openMods")}</button>
      <button type="button" class="btn btn-ghost" id="btn-reload-content">${t("common.refresh")}</button>
    </div>
    <div class="chips tiny" id="content-filters"></div>
    <div class="content-tools">
      <button type="button" class="linkish" id="btn-enable-all">${t("content.enableAll")}</button>
      <button type="button" class="linkish" id="btn-disable-all">${t("content.disableAll")}</button>
      <span class="muted" id="content-count"></span>
    </div>
    <div id="content-table" class="content-table"></div>
  `;

  panel.querySelector("#btn-browse-content")?.addEventListener("click", () => {
    browseProjectType = "mod";
    browseInstanceId = inst.id;
    routeTo("browse", inst.id);
  });

  const filters = panel.querySelector<HTMLElement>("#content-filters")!;
  const table = panel.querySelector<HTMLElement>("#content-table")!;
  const search = panel.querySelector<HTMLInputElement>("#content-search")!;
  const count = panel.querySelector<HTMLElement>("#content-count")!;
  let items: ContentItem[] = [];

  const filterDefs: Array<{ id: typeof contentFilter; label: string }> = [
    { id: "all", label: t("content.filter.all") },
    { id: "mod", label: t("content.filter.mods") },
    { id: "resourcepack", label: t("content.filter.resourcepacks") },
    { id: "shader", label: t("content.filter.shaders") },
    { id: "datapack", label: t("content.filter.datapacks") },
    { id: "enabled", label: t("content.filter.enabled") },
    { id: "disabled", label: t("content.filter.disabled") },
  ];

  const paintFilters = () => {
    filters.replaceChildren();
    for (const f of filterDefs) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `chip${contentFilter === f.id ? " is-selected" : ""}`;
      btn.textContent = f.label;
      btn.addEventListener("click", () => {
        contentFilter = f.id;
        paint();
      });
      filters.append(btn);
    }
  };

  const load = async () => {
    if (!inTauri) {
      items = inst.content;
      paint();
      return;
    }
    try {
      const rows = await invoke<
        Array<{
          id: string;
          name: string;
          kind: ContentKind;
          fileName: string;
          enabled: boolean;
          size: number;
        }>
      >("list_content", { id: inst.id });
      items = rows.map((r) => ({
        id: r.id,
        name: r.name,
        author: "disk",
        kind: r.kind,
        version: "—",
        fileName: r.fileName,
        enabled: r.enabled,
      }));
      inst.content = items;
      saveInstances();
    } catch (e) {
      topStatus.textContent = friendlyError(e);
      items = [];
    }
    paint();
  };

  const paint = () => {
    paintFilters();
    table.replaceChildren();
    const q = search.value.trim().toLowerCase();
    const rows = items.filter((item) => {
      if (
        contentFilter === "mod" ||
        contentFilter === "resourcepack" ||
        contentFilter === "shader" ||
        contentFilter === "datapack"
      ) {
        if (item.kind !== contentFilter) return false;
      } else if (contentFilter === "enabled" && !item.enabled) return false;
      else if (contentFilter === "disabled" && item.enabled) return false;
      if (!q) return true;
      return (
        item.name.toLowerCase().includes(q) || item.fileName.toLowerCase().includes(q)
      );
    });
    count.textContent = `${rows.length} / ${items.length}`;
    search.placeholder = t("content.searchN", { n: items.length });

    if (!items.length) {
      table.innerHTML = `
        <div class="empty-state">
          <h3>${t("content.emptyTitle")}</h3>
          <p class="muted">${t("content.emptyHintBrowse")}</p>
          <button type="button" class="btn btn-primary" id="content-empty-browse">${t("content.browseContent")}</button>
        </div>`;
      table.querySelector("#content-empty-browse")?.addEventListener("click", () => {
        browseProjectType = "mod";
        browseInstanceId = inst.id;
        routeTo("browse", inst.id);
      });
      return;
    }
    if (!rows.length) {
      table.innerHTML = `<p class="muted">${t("content.noMatch")}</p>`;
      return;
    }

    const head = document.createElement("div");
    head.className = "content-head";
    head.innerHTML = `<span>${t("content.colProject")}</span><span>${t("content.colFile")}</span><span>${t("content.colActions")}</span>`;
    table.append(head);

    for (const item of rows) {
      const row = document.createElement("div");
      row.className = `content-row${item.enabled ? "" : " is-disabled"}`;
      row.innerHTML = `
        <div class="content-project">
          <span class="content-icon"></span>
          <div>
            <strong></strong>
            <small></small>
          </div>
        </div>
        <div class="content-version">
          <strong></strong>
          <small></small>
        </div>
        <div class="content-actions">
          <label class="switch" title="${t("content.toggleTitle")}">
            <input type="checkbox" />
            <span></span>
          </label>
          <button type="button" class="icon-btn sm" data-del title="${t("common.delete")}">🗑</button>
        </div>
      `;
      row.querySelector(".content-icon")!.textContent = kindLabel(item.kind).slice(0, 1);
      row.querySelector(".content-project strong")!.textContent = item.name;
      row.querySelector(".content-project small")!.textContent =
        `${kindLabel(item.kind)}${item.author && item.author !== "disk" ? ` · ${item.author}` : ""}`;
      row.querySelector(".content-version strong")!.textContent = item.fileName;
      row.querySelector(".content-version small")!.textContent = item.enabled
        ? t("content.enabled")
        : t("content.disabled");
      const delBtn = row.querySelector<HTMLButtonElement>("[data-del]")!;
      delBtn.setAttribute("aria-label", t("content.deleteAria", { name: item.name }));
      const toggle = row.querySelector<HTMLInputElement>("input[type=checkbox]")!;
      toggle.checked = item.enabled;
      toggle.addEventListener("change", () => {
        void (async () => {
          if (!inTauri) {
            item.enabled = toggle.checked;
            saveInstances();
            paint();
            return;
          }
          try {
            await invoke("set_content_enabled", {
              id: inst.id,
              kind: item.kind,
              fileName: item.fileName,
              enabled: toggle.checked,
            });
            await load();
          } catch (e) {
            toggle.checked = item.enabled;
            topStatus.textContent = friendlyError(e);
          }
        })();
      });
      row.querySelector("[data-del]")?.addEventListener("click", () => {
        void (async () => {
          if (!inTauri) return;
          try {
            await invoke("delete_content", {
              id: inst.id,
              kind: item.kind,
              fileName: item.fileName,
            });
            await load();
          } catch (e) {
            topStatus.textContent = friendlyError(e);
          }
        })();
      });
      table.append(row);
    }
  };

  const setAll = async (enabled: boolean) => {
    if (!inTauri) return;
    for (const item of items) {
      if (item.enabled === enabled) continue;
      try {
        await invoke("set_content_enabled", {
          id: inst.id,
          kind: item.kind,
          fileName: item.fileName,
          enabled,
        });
      } catch (e) {
        topStatus.textContent = friendlyError(e);
        break;
      }
    }
    await load();
  };

  const addFiles = async () => {
    if (!inTauri) {
      topStatus.textContent = t("content.addBrowser");
      return;
    }
    try {
      const selected = await openDialog({
        multiple: true,
        title: t("content.addDialogTitle"),
        filters: [
          { name: "Minecraft content", extensions: ["jar", "zip"] },
          { name: "All", extensions: ["*"] },
        ],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      let added = 0;
      for (const path of paths) {
        if (typeof path !== "string" || !path) continue;
        try {
          await invoke("add_content_from_path", { id: inst.id, path, kind: null });
          added += 1;
        } catch (e) {
          topStatus.textContent = friendlyError(e);
        }
      }
      if (added > 0) {
        topStatus.textContent =
          added === 1 ? t("content.added1") : t("content.addedN", { n: added });
      }
      await load();
    } catch (e) {
      topStatus.textContent = friendlyError(e);
    }
  };

  search.addEventListener("input", paint);
  panel.querySelector("[data-open-mods]")?.addEventListener("click", () => {
    if (!inTauri) {
      setTitlebarMessage(t("files.openBrowser"));
      return;
    }
    void invoke("open_instance_dir", { id: inst.id, which: "mods" }).catch((e) => {
      setTitlebarError(friendlyError(e));
    });
  });
  panel.querySelector("#btn-add-content")?.addEventListener("click", () => void addFiles());
  panel.querySelector("#btn-enable-all")?.addEventListener("click", () => void setAll(true));
  panel.querySelector("#btn-disable-all")?.addEventListener("click", () => void setAll(false));
  panel.querySelector("#btn-reload-content")?.addEventListener("click", () => void load());
  void load();
}

function renderSharePanel(panel: HTMLElement, inst: Instance) {
  panel.innerHTML = `
    <div class="empty-state">
      <h3>${t("share.title")}</h3>
      <p class="muted">${t("share.body")}</p>
      <p class="muted tiny">${t("share.mrpackHint")}</p>
      <p class="muted">${inst.name} · ${prettyLoader(inst.loader)} ${inst.minecraft}</p>
      <div class="home-quick-actions">
        <button type="button" class="btn btn-primary" id="share-export-mrpack">${t("share.exportMrpack")}</button>
        <button type="button" class="btn btn-ghost" id="share-copy-name">${t("share.copyName")}</button>
        <button type="button" class="btn btn-ghost" id="share-open-folder">${t("instance.openFolder")}</button>
      </div>
    </div>
  `;
  panel.querySelector("#share-export-mrpack")?.addEventListener("click", () => {
    if (!inTauri) {
      setTitlebarMessage(t("share.exportBrowser"));
      return;
    }
    void (async () => {
      try {
        const dest = await saveDialog({
          title: t("share.exportMrpack"),
          defaultPath: `${inst.name.replace(/[^\w\- ]+/g, "_")}.mrpack`,
          filters: [{ name: "mrpack", extensions: ["mrpack"] }],
        });
        if (typeof dest !== "string") return;
        const msg = await invoke<string>("export_mrpack", {
          instanceId: inst.id,
          destPath: dest,
        });
        setTitlebarMessage(msg);
      } catch (e) {
        setTitlebarError(friendlyError(e));
      }
    })();
  });
  panel.querySelector("#share-copy-name")?.addEventListener("click", () => {
    void navigator.clipboard?.writeText(inst.name).then(
      () => setTitlebarMessage(t("share.copied")),
      () => setTitlebarMessage(inst.name),
    );
  });
  panel.querySelector("#share-open-folder")?.addEventListener("click", () => {
    if (!inTauri) {
      setTitlebarMessage(t("files.openBrowser"));
      return;
    }
    void invoke("open_instance_dir", { id: inst.id, which: "root" }).catch((e) => {
      setTitlebarError(friendlyError(e));
    });
  });
}

function renderFilesPanel(panel: HTMLElement, inst: Instance) {
  panel.innerHTML = `
    <div class="toolbar">
      <button type="button" class="btn btn-ghost" data-open="root">${t("files.openRoot")}</button>
      <button type="button" class="btn btn-ghost" data-open="game">${t("files.openGame")}</button>
      <button type="button" class="btn btn-ghost" data-open="mods">${t("content.openMods")}</button>
      <button type="button" class="btn btn-primary" id="files-refresh">${t("common.refresh")}</button>
    </div>
    <p class="muted" id="files-path"></p>
    <div id="files-list" class="content-table"></div>
  `;
  const list = panel.querySelector<HTMLElement>("#files-list")!;
  const pathEl = panel.querySelector<HTMLElement>("#files-path")!;
  let rel = "";
  let loadSeq = 0;

  const openWhich = async (which: string) => {
    if (!inTauri) {
      topStatus.textContent = t("files.openBrowser");
      return;
    }
    try {
      await invoke("open_instance_dir", { id: inst.id, which });
    } catch (e) {
      topStatus.textContent = String(e);
    }
  };

  panel.querySelectorAll<HTMLButtonElement>("[data-open]").forEach((btn) => {
    btn.addEventListener("click", () => void openWhich(btn.dataset.open || "game"));
  });

  const paint = async () => {
    const seq = ++loadSeq;
    pathEl.textContent = rel ? `game/${rel}` : "game/";
    list.innerHTML = `<p class="muted">${t("instance.files.loading")}</p>`;
    if (!inTauri) {
      list.innerHTML = `<p class="muted">${t("files.browser")}</p>`;
      return;
    }
    try {
      const entries = await invoke<
        Array<{ name: string; path: string; isDir: boolean; size?: number | null }>
      >("list_instance_dir", { id: inst.id, relative: rel || null });
      if (seq !== loadSeq) return;
      list.replaceChildren();
      if (!entries.length) {
        list.innerHTML = `<div class="empty-state"><h3>${t("files.emptyTitle")}</h3><p class="muted">${t("files.emptyHint")}</p></div>`;
        return;
      }
      if (rel) {
        const up = document.createElement("button");
        up.type = "button";
        up.className = "content-row";
        up.style.width = "100%";
        up.textContent = "↑ ..";
        up.addEventListener("click", () => {
          const parts = rel.split("/").filter(Boolean);
          parts.pop();
          rel = parts.join("/");
          void paint();
        });
        list.append(up);
      }
      const frag = document.createDocumentFragment();
      for (const entry of entries) {
        const row = document.createElement("button");
        row.type = "button";
        row.className = "content-row";
        row.style.width = "100%";
        row.style.textAlign = "left";
        const size =
          entry.isDir || entry.size == null
            ? ""
            : entry.size < 1024
              ? `${entry.size} B`
              : entry.size < 1024 * 1024
                ? `${Math.round(entry.size / 1024)} KB`
                : `${(entry.size / (1024 * 1024)).toFixed(1)} MB`;
        row.innerHTML = `<strong></strong> <span class="muted"></span>`;
        row.querySelector("strong")!.textContent = `${entry.isDir ? "📁 " : "📄 "}${entry.name}`;
        row.querySelector("span")!.textContent = size;
        row.addEventListener("click", () => {
          if (entry.isDir) {
            rel = entry.path;
            void paint();
          }
        });
        frag.append(row);
      }
      list.append(frag);
    } catch (e) {
      if (seq !== loadSeq) return;
      list.replaceChildren();
      const err = document.createElement("p");
      err.className = "muted";
      err.textContent = String(e);
      list.append(err);
    }
  };

  panel.querySelector("#files-refresh")?.addEventListener("click", () => void paint());
  void paint();
}

type LogSource = {
  id: string;
  name: string;
  source: string;
  filename: string;
  size: number;
  modifiedAt: number;
  live: boolean;
};

function renderLogsPanel(panel: HTMLElement, inst?: Instance) {
  panel.innerHTML = `
    <div class="logs-toolbar">
      <select id="logs-source" class="logs-select" aria-label="${t("instance.logs.source")}"></select>
      <select id="logs-level" class="logs-select logs-select-sm" aria-label="${t("instance.logs.filter")}">
        <option value="all">${t("instance.logs.filterAll")}</option>
        <option value="error">${t("instance.logs.filterError")}</option>
        <option value="warn">${t("instance.logs.filterWarn")}</option>
        <option value="info">${t("instance.logs.filterInfo")}</option>
      </select>
      <input class="search logs-search" id="logs-search" placeholder="${t("instance.logs.search")}" />
      <button type="button" class="btn btn-ghost" id="logs-open">${t("instance.logs.openFolder")}</button>
      <button type="button" class="btn btn-ghost" id="logs-refresh">${t("instance.logs.refresh")}</button>
    </div>
    <div id="logs-console" class="logs-console" role="log" aria-live="polite"></div>
  `;
  const sourceEl = panel.querySelector<HTMLSelectElement>("#logs-source")!;
  const levelEl = panel.querySelector<HTMLSelectElement>("#logs-level")!;
  const searchEl = panel.querySelector<HTMLInputElement>("#logs-search")!;
  const consoleEl = panel.querySelector<HTMLElement>("#logs-console")!;

  let sources: LogSource[] = [];
  let rawText = "";
  let selectedId = "live";
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let disposed = false;
  let loadSeq = 0;

  const stopPoll = () => {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  };

  const lineLevel = (line: string): "error" | "warn" | "info" | "other" => {
    const lower = line.toLowerCase();
    if (
      lower.includes("/error]") ||
      lower.includes("[error]") ||
      lower.includes(" exception") ||
      lower.includes("caused by:") ||
      /\berror\b/.test(lower)
    ) {
      return "error";
    }
    if (lower.includes("/warn]") || lower.includes("[warn]") || /\bwarn(ing)?\b/.test(lower)) {
      return "warn";
    }
    if (lower.includes("/info]") || lower.includes("[info]") || /\binfo\b/.test(lower)) {
      return "info";
    }
    return "other";
  };

  const paintConsole = () => {
    const q = searchEl.value.trim().toLowerCase();
    const level = levelEl.value;
    const lines = rawText.split(/\r?\n/);
    const frag = document.createDocumentFragment();
    let shown = 0;
    const maxPaint = 4_000;
    for (const line of lines) {
      if (!line && lines.length > 1) continue;
      const lvl = lineLevel(line);
      if (level === "error" && lvl !== "error") continue;
      if (level === "warn" && lvl !== "warn" && lvl !== "error") continue;
      if (level === "info" && lvl !== "info") continue;
      if (q && !line.toLowerCase().includes(q)) continue;
      const row = document.createElement("div");
      row.className = `log-line log-${lvl}`;
      row.textContent = line || " ";
      frag.append(row);
      shown += 1;
      if (shown >= maxPaint) break;
    }
    consoleEl.replaceChildren(frag);
    if (!shown) {
      const empty = document.createElement("div");
      empty.className = "logs-empty";
      if (selectedId === "live" && !rawText.trim()) {
        empty.innerHTML = `<h3>${t("instance.logs.emptyTitle")}</h3><p class="muted">${t("instance.logs.emptyHint")}</p>`;
      } else {
        empty.innerHTML = `<p class="muted">${t("instance.logs.noMatches")}</p>`;
      }
      consoleEl.append(empty);
    } else {
      consoleEl.scrollTop = consoleEl.scrollHeight;
    }
  };

  const fillSources = () => {
    const prev = selectedId;
    sourceEl.replaceChildren();
    for (const s of sources) {
      const opt = document.createElement("option");
      opt.value = s.id;
      opt.textContent = s.name;
      sourceEl.append(opt);
    }
    if (sources.some((s) => s.id === prev)) {
      sourceEl.value = prev;
      selectedId = prev;
    } else {
      sourceEl.value = sources[0]?.id ?? "live";
      selectedId = sourceEl.value;
    }
  };

  const loadSelected = async (opts?: { quiet?: boolean }) => {
    if (!inst || !inTauri) {
      rawText = "";
      paintConsole();
      return;
    }
    const seq = ++loadSeq;
    const current = sources.find((s) => s.id === selectedId) ?? sources[0];
    if (!current) {
      rawText = "";
      paintConsole();
      return;
    }
    if (!opts?.quiet) {
      consoleEl.innerHTML = `<p class="muted">${t("instance.logs.loading")}</p>`;
    }
    try {
      const text = await invoke<string>("read_instance_log", {
        packId: inst.id,
        source: current.source,
        filename: current.filename,
        maxBytes: 512_000,
      });
      if (disposed || seq !== loadSeq) return;
      rawText = text;
      paintConsole();
    } catch (e) {
      if (disposed || seq !== loadSeq) return;
      if (current.live) {
        rawText = "";
        paintConsole();
      } else {
        consoleEl.replaceChildren();
        const err = document.createElement("p");
        err.className = "muted";
        err.textContent = String(e);
        consoleEl.append(err);
      }
    }
  };

  const refreshList = async () => {
    if (!inst) {
      sources = [{ id: "live", name: "Live Log", source: "live", filename: "", size: 0, modifiedAt: 0, live: true }];
      fillSources();
      rawText = "";
      paintConsole();
      return;
    }
    if (!inTauri) {
      consoleEl.innerHTML = `<p class="muted">${t("instance.logs.browser")}</p>`;
      return;
    }
    try {
      sources = await invoke<LogSource[]>("list_instance_logs", { packId: inst.id });
      if (!sources.length) {
        sources = [{ id: "live", name: "Live Log", source: "live", filename: "", size: 0, modifiedAt: 0, live: true }];
      }
      fillSources();
      await loadSelected();
    } catch (e) {
      consoleEl.replaceChildren();
      const err = document.createElement("p");
      err.className = "muted";
      err.textContent = String(e);
      consoleEl.append(err);
    }
  };

  const ensurePoll = () => {
    stopPoll();
    const current = sources.find((s) => s.id === selectedId);
    if (!current?.live || !inst || !inTauri) return;
    pollTimer = setInterval(() => {
      void loadSelected({ quiet: true });
    }, 1200);
  };

  panel.querySelector("#logs-open")?.addEventListener("click", () => {
    if (!inTauri || !inst) {
      setTitlebarMessage(t("files.openBrowser"));
      return;
    }
    void invoke("open_instance_dir", {
      id: inst.id,
      which: "logs",
    }).catch((e) => {
      topStatus.textContent = String(e);
    });
  });

  panel.querySelector("#logs-refresh")?.addEventListener("click", () => {
    void refreshList().then(() => ensurePoll());
  });

  sourceEl.addEventListener("change", () => {
    selectedId = sourceEl.value;
    void loadSelected().then(() => ensurePoll());
  });
  levelEl.addEventListener("change", () => paintConsole());
  searchEl.addEventListener("input", () => paintConsole());

  // Cleanup when panel is replaced (tab switch rewrites innerHTML of parent).
  const observer = new MutationObserver(() => {
    if (!panel.isConnected || !panel.contains(consoleEl)) {
      disposed = true;
      stopPoll();
      observer.disconnect();
    }
  });
  observer.observe(panel.parentElement ?? document.body, { childList: true, subtree: true });

  void refreshList().then(() => ensurePoll());
}

function renderWorldsPanel(panel: HTMLElement, inst: Instance) {
  panel.innerHTML = `
    <div class="toolbar">
      <input class="search" id="world-search" placeholder="${t("worlds.search")}" />
      <button type="button" class="btn btn-ghost" id="btn-open-saves">${t("worlds.openSaves")}</button>
      <button type="button" class="btn btn-primary" id="btn-reload-worlds">${t("common.refresh")}</button>
    </div>
    <div id="world-list" class="world-list"></div>
  `;
  const list = panel.querySelector<HTMLElement>("#world-list")!;
  const search = panel.querySelector<HTMLInputElement>("#world-search")!;
  let worlds: WorldItem[] = [];

  const paint = () => {
    list.replaceChildren();
    const q = search.value.trim().toLowerCase();
    const rows = worlds.filter((w) => !q || w.name.toLowerCase().includes(q));
    if (!worlds.length) {
      list.innerHTML = `
        <div class="empty-state">
          <h3>${t("worlds.emptyTitle")}</h3>
          <p class="muted">${t("worlds.emptyHint")}</p>
        </div>`;
      return;
    }
    if (!rows.length) {
      list.innerHTML = `<p class="muted">${t("worlds.noMatch")}</p>`;
      return;
    }
    for (const world of rows) {
      const row = document.createElement("div");
      row.className = "world-row";
      row.innerHTML = `
        <div class="world-icon"></div>
        <div class="world-meta">
          <strong></strong>
          <small></small>
        </div>
        <button type="button" class="icon-btn sm" data-del title="${t("worlds.deleteTitle")}">🗑</button>
      `;
      const delWorld = row.querySelector<HTMLButtonElement>("[data-del]")!;
      delWorld.setAttribute("aria-label", t("worlds.deleteAria", { name: world.name }));
      const iconEl = row.querySelector<HTMLElement>(".world-icon")!;
      if (world.icon) {
        iconEl.classList.add("has-image");
        const img = document.createElement("img");
        img.src = world.icon;
        img.alt = "";
        iconEl.append(img);
      } else {
        iconEl.textContent = "♣";
      }
      row.querySelector("strong")!.textContent = world.name;
      row.querySelector("small")!.textContent =
        `${world.mode} · ${new Date(world.lastPlayedAt).toLocaleString()}`;
      row.querySelector("[data-del]")?.addEventListener("click", () => {
        void (async () => {
          if (!inTauri) return;
          try {
            await invoke("delete_world", { id: inst.id, worldId: world.id });
            await load();
          } catch (e) {
            topStatus.textContent = friendlyError(e);
          }
        })();
      });
      list.append(row);
    }
  };

  const load = async () => {
    if (!inTauri) {
      worlds = inst.worlds;
      paint();
      return;
    }
    try {
      const rows = await invoke<
        Array<{
          id: string;
          name: string;
          mode: string;
          lastPlayedAt: number;
          hardcore?: boolean;
          icon?: string | null;
        }>
      >("list_worlds", { id: inst.id });
      worlds = rows.map((w) => ({
        id: w.id,
        name: w.name,
        mode: w.hardcore ? "Hardcore" : w.mode,
        lastPlayedAt: w.lastPlayedAt < 1e12 ? w.lastPlayedAt * 1000 : w.lastPlayedAt,
        hardcore: Boolean(w.hardcore),
        icon: w.icon ?? null,
      }));
      inst.worlds = worlds;
      saveInstances();
    } catch (e) {
      topStatus.textContent = friendlyError(e);
      worlds = [];
    }
    paint();
  };

  search.addEventListener("input", paint);
  panel.querySelector("#btn-open-saves")?.addEventListener("click", () => {
    if (!inTauri) {
      setTitlebarMessage(t("files.openBrowser"));
      return;
    }
    void invoke("open_instance_dir", { id: inst.id, which: "saves" }).catch((e) => {
      setTitlebarError(friendlyError(e));
    });
  });
  panel.querySelector("#btn-reload-worlds")?.addEventListener("click", () => void load());
  void load();
}

function friendlyError(e: unknown): string {
  const raw = String(e);
  if (/install is already running/i.test(raw)) return t("error.installRunning");
  if (/Cannot delete instance while install/i.test(raw)) return t("error.cantDeleteInstalling");
  if (/Cannot duplicate while install/i.test(raw)) return t("error.cantDupInstalling");
  if (/(Download host not allowed|host not allowed|Only https)/i.test(raw)) return t("error.hostNotAllowed");
  if (/(File too large|too large)/i.test(raw)) return t("error.fileTooLarge");
  if (/Pack is not ready/i.test(raw)) return t("error.notReady");
  if (/Content file already exists/i.test(raw)) return t("error.contentExists");
  if (/Unable to infer content type/i.test(raw)) return t("error.inferType");
  if (/Source file not found/i.test(raw)) return t("error.sourceNotFound");
  // Modrinth install (MR-1)
  if (/No compatible version/i.test(raw)) return t("error.noCompatVersion");
  if (/SHA-?512 verification/i.test(raw)) return t("error.hashFail");
  if (/no downloadable file/i.test(raw)) return t("error.noFile");
  // Launch / install / create
  if (/Java \d+\+? not found/i.test(raw) || /Java .* -version probe/i.test(raw)) return t("error.javaMissing");
  if (/Nick (is required|may only)/i.test(raw)) return t("error.nickInvalid");
  if (/Minecraft version is required/i.test(raw)) return t("error.mcRequired");
  if (/Loader version is required/i.test(raw)) return t("error.loaderRequired");
  if (/Unsupported loader|is not supported/i.test(raw)) return t("error.unsupportedLoader");
  if (/(module-path|--launchTarget|installer (failed|timed out))/i.test(raw)) return t("error.modLoaderInstall");
  if (/No \.jar files in manifest/i.test(raw)) return t("error.noJars");
  if (/(timed out|timeout)/i.test(raw)) return t("error.timeout");
  if (/(SHA1 mismatch|Size mismatch|integrity check failed)/i.test(raw)) return t("error.integrity");
  if (/error sending request|dns error|connect|network|failed to lookup/i.test(raw)) return t("error.network");
  return raw.replace(/^Error:\s*/i, "").slice(0, 240);
}

async function duplicateInstanceOnDisk(inst: Instance): Promise<void> {
  if (inst.installStatus === "installing") {
    setTitlebarIdle(t("instance.waitInstall"));
    return;
  }
  const newId = `inst_${Date.now().toString(36)}`;
  const newName = `${inst.name} (copy)`.slice(0, 48);
  if (!inTauri) {
    const copy = normalizeInstance({
      ...structuredClone(inst),
      id: newId,
      name: newName,
      createdAt: Date.now(),
      lastPlayedAt: undefined,
    });
    instances = [copy, ...instances];
    saveInstances();
    selectedInstanceId = copy.id;
    routeTo("instance", copy.id);
    return;
  }
  try {
    setTitlebarMessage(t("instance.copying"));
    const disk = await invoke<DiskInstance>("duplicate_instance", {
      id: inst.id,
      newId,
      newName,
    });
    await hydrateInstancesFromDisk();
    const found = instances.find((i) => i.id === disk.id);
    if (found) {
      found.settings = { ...structuredClone(inst.settings) };
      found.name = newName;
      saveInstances();
    }
    selectedInstanceId = disk.id;
    setTitlebarIdle(t("instance.copied"));
    routeTo("instance", disk.id);
  } catch (e) {
    setTitlebarMessage(friendlyError(e));
  }
}

function renderInstanceSettings(inst: Instance) {
  showModal(modalInstSettings);
  modalInstSettings.querySelector("#inst-settings-title")!.textContent =
    `${inst.name} · ${t("instance.settings.crumb")}`;
  const nav = modalInstSettings.querySelector<HTMLElement>("#inst-settings-nav")!;
  const pane = modalInstSettings.querySelector<HTMLElement>("#inst-settings-pane")!;
  let installEditing = false;

  const tabs: Array<{ id: typeof instanceSettingsTab; label: string }> = [
    { id: "general", label: t("instance.settings.general") },
    { id: "install", label: t("instance.settings.install") },
    { id: "window", label: t("instance.settings.window") },
    { id: "java", label: t("instance.settings.java") },
    { id: "launch", label: t("instance.settings.launch") },
  ];

  const paintNav = () => {
    nav.replaceChildren();
    for (const tab of tabs) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `settings-link${instanceSettingsTab === tab.id ? " is-active" : ""}`;
      btn.textContent = tab.label;
      btn.addEventListener("click", () => {
        instanceSettingsTab = tab.id;
        installEditing = false;
        void paint();
      });
      nav.append(btn);
    }
  };

  const persistSettings = () => {
    saveInstances();
  };

  const paint = async () => {
    paintNav();
    pane.replaceChildren();
    if (instanceSettingsTab === "general") {
      const groups = [
        ...new Set(
          instances
            .map((i) => i.settings.group.trim())
            .filter(Boolean)
            .concat(inst.settings.group.trim() ? [inst.settings.group.trim()] : []),
        ),
      ].sort((a, b) => a.localeCompare(b));
      pane.innerHTML = `
        <h2>${t("instance.settings.general")}</h2>
        <div class="iset-name-row">
          <div class="instance-icon lg iset-icon" id="iset-icon"></div>
          <div class="iset-icon-actions">
            <button type="button" class="btn btn-ghost" id="iset-pick-icon">${t("create.selectIcon")}</button>
            <button type="button" class="btn btn-ghost" id="iset-clear-icon" ${inst.hasIcon || inst.iconUrl ? "" : "disabled"}>${t("create.removeIcon")}</button>
          </div>
          <div class="field grow">
            <label for="rename">${t("instance.settings.name")}</label>
            <input id="rename" maxlength="48" />
          </div>
        </div>
        <div class="modal-actions">
          <button type="button" class="btn btn-ghost" id="dup-inst">${t("instance.dup")}</button>
        </div>
        <div class="field">
          <span class="label">${t("instance.settings.group")}</span>
          <p class="hint">${t("instance.settings.groupHint")}</p>
          <div class="chips" id="group-chips"></div>
          <div class="field-row" style="margin-top:8px">
            <input id="new-group" maxlength="32" placeholder="${t("instance.settings.newGroup")}" />
            <button type="button" class="btn btn-ghost" id="add-group">${t("instance.settings.createGroup")}</button>
          </div>
        </div>
        <div class="field">
          <span class="label">${t("instance.settings.channel")}</span>
          <div class="chips" id="channel-chips"></div>
          <p class="hint" id="channel-hint"></p>
        </div>
        <div class="danger-zone">
          <button type="button" class="btn btn-danger" id="delete-inst">${t("instance.settings.delete")}</button>
        </div>
        <p id="iset-status" class="status"></p>
      `;
      const rename = pane.querySelector<HTMLInputElement>("#rename")!;
      const status = pane.querySelector<HTMLElement>("#iset-status")!;
      rename.value = inst.name;
      applyInstanceIcon(pane.querySelector("#iset-icon")!, inst);
      rename.addEventListener("change", () => {
        inst.name = (rename.value.trim() || inst.name).slice(0, 48);
        persistSettings();
        setStatus(status, t("instance.settings.saved"), "ok");
        renderRecent();
        applyInstanceIcon(pane.querySelector("#iset-icon")!, inst);
      });
      pane.querySelector("#iset-pick-icon")?.addEventListener("click", () => {
        void (async () => {
          if (!inTauri) {
            setStatus(status, t("play.browser"), "muted");
            return;
          }
          const picked = await openDialog({
            multiple: false,
            filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }],
          });
          const path = typeof picked === "string" ? picked : Array.isArray(picked) ? picked[0] : null;
          if (!path) return;
          try {
            await invoke("set_instance_icon", { id: inst.id, path });
            const url = await invoke<string | null>("get_instance_icon", { id: inst.id });
            inst.hasIcon = Boolean(url);
            inst.iconUrl = url;
            applyInstanceIcon(pane.querySelector("#iset-icon")!, inst);
            const clearBtn = pane.querySelector<HTMLButtonElement>("#iset-clear-icon");
            if (clearBtn) clearBtn.disabled = !inst.hasIcon;
            setStatus(status, t("instance.settings.saved"), "ok");
            if (selectedInstanceId === inst.id) renderInstance();
            renderRecent();
          } catch (e) {
            setStatus(status, friendlyError(e), "err");
          }
        })();
      });
      pane.querySelector("#iset-clear-icon")?.addEventListener("click", () => {
        void (async () => {
          if (!inTauri) return;
          try {
            await invoke("set_instance_icon", { id: inst.id, path: null });
            inst.hasIcon = false;
            inst.iconUrl = null;
            applyInstanceIcon(pane.querySelector("#iset-icon")!, inst);
            const clearBtn = pane.querySelector<HTMLButtonElement>("#iset-clear-icon");
            if (clearBtn) clearBtn.disabled = true;
            setStatus(status, t("instance.settings.saved"), "ok");
            if (selectedInstanceId === inst.id) renderInstance();
            renderRecent();
          } catch (e) {
            setStatus(status, friendlyError(e), "err");
          }
        })();
      });

      const groupBox = pane.querySelector<HTMLElement>("#group-chips")!;
      const paintGroups = () => {
        groupBox.replaceChildren();
        for (const g of groups) {
          const btn = document.createElement("button");
          btn.type = "button";
          btn.className = `chip${inst.settings.group === g ? " is-selected" : ""}`;
          btn.textContent = (inst.settings.group === g ? "✓ " : "") + g;
          btn.addEventListener("click", () => {
            inst.settings.group = inst.settings.group === g ? "" : g;
            persistSettings();
            paintGroups();
          });
          groupBox.append(btn);
        }
      };
      paintGroups();
      pane.querySelector("#add-group")?.addEventListener("click", () => {
        const input = pane.querySelector<HTMLInputElement>("#new-group")!;
        const name = input.value.trim().slice(0, 32);
        if (!name) return;
        if (!groups.includes(name)) groups.push(name);
        inst.settings.group = name;
        input.value = "";
        persistSettings();
        paintGroups();
      });

      const channelHint = pane.querySelector<HTMLElement>("#channel-hint")!;
      const channelBox = pane.querySelector<HTMLElement>("#channel-chips")!;
      const channelDesc: Record<string, string> = {
        release: t("instance.settings.channel.release"),
        beta: t("instance.settings.channel.beta"),
        alpha: t("instance.settings.channel.alpha"),
      };
      for (const ch of ["release", "beta", "alpha"] as const) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = `chip${inst.settings.updateChannel === ch ? " is-selected" : ""}`;
        btn.textContent =
          (inst.settings.updateChannel === ch ? "✓ " : "") +
          ch[0]!.toUpperCase() +
          ch.slice(1);
        btn.addEventListener("click", () => {
          inst.settings.updateChannel = ch;
          persistSettings();
          channelHint.textContent = channelDesc[ch] ?? "";
          void paint();
        });
        channelBox.append(btn);
      }
      channelHint.textContent = channelDesc[inst.settings.updateChannel] ?? "";

      pane.querySelector("#dup-inst")?.addEventListener("click", () => {
        void duplicateInstanceOnDisk(inst);
      });
      pane.querySelector("#delete-inst")?.addEventListener("click", () => {
        void deleteInstanceEverywhere(inst);
      });
    } else if (instanceSettingsTab === "install") {
      if (!installEditing) {
        pane.innerHTML = `
          <h2>${t("instance.settings.install")}</h2>
          <div class="install-details card-inset">
            <div class="info-grid">
              <div><span class="muted">${t("instance.settings.platform")}</span><strong id="i-loader"></strong></div>
              <div><span class="muted">${t("instance.settings.minecraft")}</span><strong id="i-mc"></strong></div>
              <div><span class="muted">${t("instance.settings.loaderVersion")}</span><strong id="i-lv"></strong></div>
            </div>
            <button type="button" class="btn btn-warn" id="btn-change-install">${t("instance.settings.change")}</button>
            <p class="hint">${t("instance.settings.changeWarn")}</p>
          </div>
          <h3>${t("instance.settings.repair")}</h3>
          <p class="hint">${t("instance.settings.repairHint")}</p>
          <button type="button" class="btn btn-ghost" id="btn-repair">${t("instance.settings.repairBtn")}</button>
          <p id="iset-status" class="status"></p>
        `;
        pane.querySelector("#i-loader")!.textContent = prettyLoader(inst.loader);
        pane.querySelector("#i-mc")!.textContent = inst.minecraft;
        pane.querySelector("#i-lv")!.textContent =
          inst.loader === "vanilla" ? "—" : inst.loaderVersion || "—";
        pane.querySelector("#btn-change-install")?.addEventListener("click", () => {
          installEditing = true;
          void paint();
        });
        pane.querySelector("#btn-repair")?.addEventListener("click", () => {
          void (async () => {
            const status = pane.querySelector<HTMLElement>("#iset-status")!;
            if (!inTauri) {
              setStatus(status, t("play.browser"), "muted");
              return;
            }
            if (!metaPrefetch.ready && !metaPrefetchFailed) {
              setStatus(status, t("meta.wait"), "err");
              return;
            }
            setStatus(status, t("instance.settings.repairing"), "muted");
            try {
              hideModal(modalInstSettings);
              await runInstall(inst);
            } catch (e) {
              setStatus(status, friendlyError(e), "err");
            }
          })();
        });
      } else {
        let editLoader = inst.loader;
        let editShowSnapshots = false;
        pane.innerHTML = `
          <h2>${t("instance.settings.changeTitle")}</h2>
          <div class="field">
            <span class="label">${t("instance.settings.platform")}</span>
            <div class="chips" id="edit-loader-chips"></div>
          </div>
          <div class="field">
            <span class="label">${t("instance.settings.minecraft")}</span>
            <div id="edit-mc-host"></div>
          </div>
          <div class="field" id="edit-lv-field">
            <span class="label" id="edit-lv-label">${t("instance.settings.loaderVersion")}</span>
            <div id="edit-lv-host"></div>
          </div>
          <div class="modal-actions">
            <button type="button" class="btn btn-primary" id="save-install">${t("instance.settings.save")}</button>
            <button type="button" class="btn btn-ghost" id="cancel-install">${t("instance.settings.cancel")}</button>
          </div>
          <p id="iset-status" class="status"></p>
        `;
        const lvField = pane.querySelector<HTMLElement>("#edit-lv-field")!;
        const lvLabel = pane.querySelector<HTMLElement>("#edit-lv-label")!;
        const status = pane.querySelector<HTMLElement>("#iset-status")!;
        const chipsBox = pane.querySelector<HTMLElement>("#edit-loader-chips")!;
        const editMcHost = pane.querySelector<HTMLElement>("#edit-mc-host")!;
        const editLvHost = pane.querySelector<HTMLElement>("#edit-lv-host")!;

        const editMcCombo = createVersionCombobox({
          searchPlaceholder: t("create.searchVersion"),
          showAllToggle: true,
          showAllLabel: t("create.showAllVersions"),
          hideAllLabel: t("create.hideSnapshots"),
          getShowAll: () => editShowSnapshots,
          onShowAllChange: (v) => {
            editShowSnapshots = v;
            void refreshEditVersions();
          },
          onChange: () => {
            void (async () => {
              if (editLoader === "vanilla") return;
              const loaders = await tauriOrBrowserLoaderVersions(
                editLoader,
                editMcCombo.getValue(),
              );
              editLvCombo.setOptions(loaders, loaders[0]);
            })();
          },
        });
        const editLvCombo = createVersionCombobox({
          searchPlaceholder: t("create.searchVersion"),
        });
        editMcHost.append(editMcCombo.root);
        editLvHost.append(editLvCombo.root);

        const refreshEditVersions = async () => {
          setStatus(status, t("create.loadingVersions"), "muted");
          lvField.hidden = editLoader === "vanilla";
          lvLabel.textContent = t("create.loaderVersion", { loader: prettyLoader(editLoader) });
          try {
            const ids = await tauriOrBrowserMcVersions(editLoader, editShowSnapshots);
            editMcCombo.setOptions(
              ids,
              ids.includes(inst.minecraft) ? inst.minecraft : ids[0],
            );
            if (editLoader === "vanilla") {
              editLvCombo.setOptions([]);
            } else {
              const loaders = await tauriOrBrowserLoaderVersions(
                editLoader,
                editMcCombo.getValue(),
              );
              editLvCombo.setOptions(
                loaders,
                loaders.includes(inst.loaderVersion) ? inst.loaderVersion : loaders[0],
              );
            }
            setStatus(status, "", "muted");
          } catch (e) {
            setStatus(status, String(e), "err");
          }
        };

        const paintLoaderChips = () => {
          chipsBox.replaceChildren();
          for (const loader of LOADERS) {
            const btn = document.createElement("button");
            btn.type = "button";
            btn.className = `chip${loader.id === editLoader ? " is-selected" : ""}`;
            btn.textContent = (loader.id === editLoader ? "✓ " : "") + loader.name;
            btn.addEventListener("click", () => {
              editLoader = loader.id;
              paintLoaderChips();
              void refreshEditVersions();
            });
            chipsBox.append(btn);
          }
        };
        paintLoaderChips();
        await refreshEditVersions();

        pane.querySelector("#cancel-install")?.addEventListener("click", () => {
          installEditing = false;
          void paint();
        });
        pane.querySelector("#save-install")?.addEventListener("click", () => {
          void (async () => {
            const minecraft = editMcCombo.getValue();
            const loaderVersion = editLoader === "vanilla" ? "" : editLvCombo.getValue();
            if (!minecraft) {
              setStatus(status, t("create.needMc"), "err");
              return;
            }
            if (editLoader !== "vanilla" && !loaderVersion) {
              setStatus(status, t("create.needLoader"), "err");
              return;
            }
            try {
              if (inTauri) {
                await invoke("update_instance_installation", {
                  id: inst.id,
                  loader: editLoader,
                  minecraft,
                  loaderVersion,
                });
              }
              inst.loader = editLoader;
              inst.minecraft = minecraft;
              inst.loaderVersion = loaderVersion;
              inst.installStatus = "pending";
              inst.installMessage = undefined;
              persistSettings();
              installEditing = false;
              setStatus(status, t("instance.settings.saved"), "ok");
              void paint();
              if (selectedInstanceId === inst.id) renderInstance();
            } catch (e) {
              setStatus(status, friendlyError(e), "err");
            }
          })();
        });
      }
    } else if (instanceSettingsTab === "window") {
      pane.innerHTML = `
        <h2>${t("instance.settings.window")}</h2>
        <div class="field-row">
          <div class="field">
            <label for="w">${t("instance.settings.width")}</label>
            <input id="w" type="number" min="640" max="7680" />
          </div>
          <div class="field">
            <label for="h">${t("instance.settings.height")}</label>
            <input id="h" type="number" min="480" max="4320" />
          </div>
        </div>
        <div class="modal-actions">
          <button type="button" class="btn btn-primary" id="save-window">${t("instance.settings.save")}</button>
        </div>
        <p id="iset-status" class="status"></p>
      `;
      const w = pane.querySelector<HTMLInputElement>("#w")!;
      const h = pane.querySelector<HTMLInputElement>("#h")!;
      const status = pane.querySelector<HTMLElement>("#iset-status")!;
      w.value = String(inst.settings.width);
      h.value = String(inst.settings.height);
      pane.querySelector("#save-window")?.addEventListener("click", () => {
        inst.settings.width = Math.min(7680, Math.max(640, Number(w.value) || 1280));
        inst.settings.height = Math.min(4320, Math.max(480, Number(h.value) || 720));
        persistSettings();
        setStatus(status, t("instance.settings.saved"), "ok");
      });
    } else if (instanceSettingsTab === "java") {
      const maxMem = Math.max(512, systemMemoryMb);
      const quickGb = [2, 3, 4, 6, 8, 10, 12, 16].filter((gb) => gb * 1024 <= maxMem);
      pane.innerHTML = `
        <h2>${t("instance.settings.javaInstall")}</h2>
        <label class="check"><input type="checkbox" id="ov-java" /><span>${t("instance.settings.overrideJava")}</span></label>
        <div class="java-path-row" id="java-path-row">
          <input id="jpath" placeholder="C:\\Program Files\\Java\\...\\javaw.exe" />
          <button type="button" class="btn btn-ghost" id="btn-browse-java">${t("settings.java.browse")}</button>
          <button type="button" class="btn btn-ghost" id="btn-detect-java-inst">${t("settings.java.detect")}</button>
        </div>
        <h2>${t("instance.settings.memory")}</h2>
        <label class="check"><input type="checkbox" id="ov-mem" /><span>${t("instance.settings.overrideMemory")}</span></label>
        <div class="memory-block" id="memory-block">
          <div class="memory-slider-wrap">
            <input id="mem-range" type="range" min="512" max="${maxMem}" step="256" />
            <div class="memory-scale">
              <span>512</span>
              <span>${maxMem}</span>
            </div>
          </div>
          <div class="memory-quick" id="memory-quick">
            ${quickGb
              .map(
                (gb) =>
                  `<button type="button" class="chip memory-quick-btn" data-mb="${gb * 1024}">${gb} GB</button>`,
              )
              .join("")}
          </div>
          <div class="memory-row">
            <input id="mem" type="number" min="512" max="${maxMem}" step="256" />
            <span class="muted">MB</span>
          </div>
        </div>
        <h2>${t("instance.settings.jvmArgs")}</h2>
        <label class="check"><input type="checkbox" id="ov-jvm" /><span>${t("instance.settings.overrideJvm")}</span></label>
        <textarea id="jvm" rows="3" placeholder="-XX:+UseG1GC"></textarea>
        <h2>${t("instance.settings.envVars")}</h2>
        <label class="check"><input type="checkbox" id="ov-env" /><span>${t("instance.settings.overrideEnv")}</span></label>
        <textarea id="env" rows="2" placeholder="KEY=value KEY2=value"></textarea>
        <p id="iset-status" class="status"></p>
      `;
      const ovJava = pane.querySelector<HTMLInputElement>("#ov-java")!;
      const ovMem = pane.querySelector<HTMLInputElement>("#ov-mem")!;
      const ovJvm = pane.querySelector<HTMLInputElement>("#ov-jvm")!;
      const ovEnv = pane.querySelector<HTMLInputElement>("#ov-env")!;
      const jpath = pane.querySelector<HTMLInputElement>("#jpath")!;
      const memRange = pane.querySelector<HTMLInputElement>("#mem-range")!;
      const mem = pane.querySelector<HTMLInputElement>("#mem")!;
      const jvm = pane.querySelector<HTMLTextAreaElement>("#jvm")!;
      const env = pane.querySelector<HTMLTextAreaElement>("#env")!;
      const status = pane.querySelector<HTMLElement>("#iset-status")!;
      ovJava.checked = inst.settings.overrideJavaPath;
      ovMem.checked = inst.settings.overrideMemory;
      ovJvm.checked = inst.settings.overrideJvmArgs;
      ovEnv.checked = inst.settings.overrideEnvVars;
      jpath.value = inst.settings.javaPath;
      const initialMem = Math.min(maxMem, Math.max(512, inst.settings.memoryMb || 4096));
      mem.value = String(initialMem);
      memRange.value = String(initialMem);
      jvm.value = inst.settings.jvmArgs;
      env.value = inst.settings.envVars;

      const syncDisabled = () => {
        jpath.disabled = !ovJava.checked;
        pane.querySelectorAll("#java-path-row button").forEach((b) => {
          (b as HTMLButtonElement).disabled = !ovJava.checked;
        });
        mem.disabled = !ovMem.checked;
        memRange.disabled = !ovMem.checked;
        pane.querySelectorAll<HTMLButtonElement>(".memory-quick-btn").forEach((b) => {
          b.disabled = !ovMem.checked;
        });
        jvm.disabled = !ovJvm.checked;
        env.disabled = !ovEnv.checked;
      };
      const paintQuick = () => {
        const cur = Number(mem.value) || 0;
        pane.querySelectorAll<HTMLButtonElement>(".memory-quick-btn").forEach((b) => {
          const mb = Number(b.dataset.mb);
          b.classList.toggle("is-selected", mb === cur);
        });
      };
      const autosaveJava = () => {
        inst.settings.overrideJavaPath = ovJava.checked;
        inst.settings.overrideMemory = ovMem.checked;
        inst.settings.overrideJvmArgs = ovJvm.checked;
        inst.settings.overrideEnvVars = ovEnv.checked;
        inst.settings.javaPath = jpath.value.trim();
        inst.settings.memoryMb = Math.min(maxMem, Math.max(512, Number(mem.value) || 4096));
        inst.settings.jvmArgs = jvm.value.slice(0, 2000);
        inst.settings.envVars = env.value.slice(0, 2000);
        mem.value = String(inst.settings.memoryMb);
        memRange.value = String(inst.settings.memoryMb);
        persistSettings();
        paintQuick();
        setStatus(status, t("instance.settings.saved"), "ok");
      };
      const setMemory = (mb: number) => {
        const clamped = Math.min(maxMem, Math.max(512, mb));
        mem.value = String(clamped);
        memRange.value = String(clamped);
        autosaveJava();
      };
      syncDisabled();
      paintQuick();
      [ovJava, ovMem, ovJvm, ovEnv].forEach((el) =>
        el.addEventListener("change", () => {
          syncDisabled();
          autosaveJava();
        }),
      );
      memRange.addEventListener("input", () => {
        mem.value = memRange.value;
        paintQuick();
        setMemory(Number(memRange.value));
      });
      mem.addEventListener("input", () => {
        memRange.value = mem.value;
        paintQuick();
      });
      mem.addEventListener("change", () => setMemory(Number(mem.value)));
      pane.querySelectorAll<HTMLButtonElement>(".memory-quick-btn").forEach((btn) => {
        btn.addEventListener("click", () => setMemory(Number(btn.dataset.mb)));
      });
      jpath.addEventListener("change", () => autosaveJava());
      jvm.addEventListener("change", () => autosaveJava());
      env.addEventListener("change", () => autosaveJava());
      pane.querySelector("#btn-browse-java")?.addEventListener("click", () => {
        void (async () => {
          const picked = await openDialog({ multiple: false });
          if (typeof picked === "string") {
            jpath.value = picked;
            autosaveJava();
          }
        })();
      });
      pane.querySelector("#btn-detect-java-inst")?.addEventListener("click", () => {
        openJavaDetectModal(17, jpath.value, (path) => {
          jpath.value = path;
          autosaveJava();
        });
      });
    } else {
      pane.innerHTML = `
        <h2>${t("instance.settings.launch")}</h2>
        <label class="check"><input type="checkbox" id="ov-hooks" /><span>${t("instance.settings.overrideHooks")}</span></label>
        <p class="hint">${t("instance.settings.hooksHint")}</p>
        <div class="field">
          <label for="pre">${t("instance.settings.preLaunch")}</label>
          <input id="pre" placeholder="${t("instance.settings.preLaunchPh")}" />
          <p class="hint">${t("instance.settings.preLaunchHint")}</p>
        </div>
        <div class="field">
          <label for="wrap">${t("instance.settings.wrapper")}</label>
          <input id="wrap" placeholder="${t("instance.settings.wrapperPh")}" />
          <p class="hint">${t("instance.settings.wrapperHint")}</p>
        </div>
        <div class="field">
          <label for="post">${t("instance.settings.postExit")}</label>
          <input id="post" placeholder="${t("instance.settings.postExitPh")}" />
          <p class="hint">${t("instance.settings.postExitHint")}</p>
        </div>
        <div class="modal-actions">
          <button type="button" class="btn btn-primary" id="save-launch">${t("instance.settings.save")}</button>
        </div>
        <p id="iset-status" class="status"></p>
      `;
      const ov = pane.querySelector<HTMLInputElement>("#ov-hooks")!;
      const pre = pane.querySelector<HTMLInputElement>("#pre")!;
      const wrap = pane.querySelector<HTMLInputElement>("#wrap")!;
      const post = pane.querySelector<HTMLInputElement>("#post")!;
      const status = pane.querySelector<HTMLElement>("#iset-status")!;
      ov.checked = inst.settings.overrideHooks;
      pre.value = inst.settings.preLaunch;
      wrap.value = inst.settings.wrapper;
      post.value = inst.settings.postExit;
      const sync = () => {
        pre.disabled = !ov.checked;
        wrap.disabled = !ov.checked;
        post.disabled = !ov.checked;
      };
      sync();
      ov.addEventListener("change", sync);
      pane.querySelector("#save-launch")?.addEventListener("click", () => {
        inst.settings.overrideHooks = ov.checked;
        inst.settings.preLaunch = pre.value.slice(0, 500);
        inst.settings.wrapper = wrap.value.slice(0, 500);
        inst.settings.postExit = post.value.slice(0, 500);
        persistSettings();
        setStatus(status, t("instance.settings.saved"), "ok");
      });
    }
  };

  void paint().then(() => {
    modalInstSettings.querySelector<HTMLElement>("#rename")?.focus();
  });
}

function openJavaDetectModal(
  major: number,
  currentPath: string,
  onSelect: (path: string) => void,
) {
  showModal(modalJavaDetect, "[data-close='java-detect']");
  modalJavaDetect.querySelector("#java-detect-title")!.textContent = t(
    "settings.java.selectTitle",
  );
  const list = modalJavaDetect.querySelector<HTMLElement>("#java-detect-list")!;
  list.innerHTML = `<p class="muted">${t("settings.java.detecting")}</p>`;
  javaDetectCallback = onSelect;

  void (async () => {
    try {
      let found: JavaInstallation[] = [];
      if (inTauri) {
        found = await invoke<JavaInstallation[]>("find_java_for_major", { major });
        if (!found.length) {
          found = await invoke<JavaInstallation[]>("detect_java");
          found = found.filter((j) => j.majorVersion === major);
        }
      }
      list.replaceChildren();
      if (!found.length) {
        list.innerHTML = `<p class="muted">${t("settings.java.noneMajor", { n: major })}</p>`;
        return;
      }
      for (const j of found) {
        const row = document.createElement("div");
        row.className = "java-detect-row";
        const selected = j.path === currentPath;
        row.innerHTML = `<div><strong></strong><code></code></div><button type="button" class="btn ${selected ? "btn-primary" : "btn-ghost"}"></button>`;
        row.querySelector("strong")!.textContent = `Java ${j.version}`;
        row.querySelector("code")!.textContent = j.path;
        const btn = row.querySelector("button")!;
        btn.textContent = selected ? t("settings.java.selected") : t("settings.java.select");
        btn.addEventListener("click", () => {
          javaDetectCallback?.(j.path);
          hideModal(modalJavaDetect);
          javaDetectCallback = null;
        });
        list.append(row);
      }
    } catch (e) {
      list.replaceChildren();
      const err = document.createElement("p");
      err.className = "status is-err";
      err.textContent = String(e);
      list.append(err);
    }
  })();
}

async function openAppSettingsModal() {

  showModal(modalAppSettings, "[data-close='app-settings']");
  modalAppSettings.querySelector("#app-settings-title")!.textContent = t("nav.settings");
  const nav = modalAppSettings.querySelector<HTMLElement>("#app-settings-nav")!;
  const pane = modalAppSettings.querySelector<HTMLElement>("#app-settings-pane")!;

  const tabs: Array<{ id: typeof settingsTab; label: string }> = [
    { id: "appearance", label: t("settings.appearance") },
    { id: "java", label: t("settings.java") },
    { id: "localization", label: t("settings.localization") },
    { id: "behavior", label: t("settings.behavior") },
    { id: "updates", label: t("settings.updates") },
  ];

  const paintNav = () => {
    nav.replaceChildren();
    for (const tab of tabs) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `settings-link${settingsTab === tab.id ? " is-active" : ""}`;
      btn.textContent = tab.label;
      btn.addEventListener("click", () => {
        settingsTab = tab.id;
        void paint();
      });
      nav.append(btn);
    }
  };

  const paint = async () => {
    paintNav();
    pane.replaceChildren();
    if (settingsTab === "appearance") {
      pane.innerHTML = `
        <h2>${t("settings.appearance")}</h2>
        <p class="muted">${t("settings.appearance.desc")}</p>
        <div class="settings-token-swatches" aria-hidden="true">
          <span style="background:#050508"></span>
          <span style="background:#131319"></span>
          <span style="background:#00e5ff"></span>
          <span style="background:#5eead4"></span>
        </div>
        <p class="muted tiny">${t("settings.appearance.tokens")}</p>
      `;
      return;
    }
    if (settingsTab === "behavior") {
      pane.innerHTML = `
        <h2>${t("settings.behavior")}</h2>
        <p class="muted">${t("settings.behavior.desc")}</p>
        <label class="check-row"><input type="checkbox" checked disabled /><span>${t("settings.behavior.closeToTray")}</span></label>
        <label class="check-row"><input type="checkbox" checked disabled /><span>${t("settings.behavior.keepLogs")}</span></label>
        <p class="muted tiny">${t("settings.behavior.stub")}</p>
      `;
      return;
    }
    if (settingsTab === "java") {
      pane.innerHTML = `
        <h2>${t("settings.java")}</h2>
        <p class="muted">${t("settings.java.desc")}</p>
        <div id="java-majors" class="java-majors"></div>
        <p id="settings-status" class="status"></p>
      `;
      const box = pane.querySelector<HTMLElement>("#java-majors")!;
      const status = pane.querySelector<HTMLElement>("#settings-status")!;
      const majors = [25, 21, 17, 8];
      const paths: Record<number, string> = {};
      for (const m of majors) {
        paths[m] = launcherCfg?.javaVersions?.[String(m)] ?? "";
      }

      const renderMajors = () => {
        box.replaceChildren();
        for (const major of majors) {
          const section = document.createElement("div");
          section.className = "java-major-block";
          section.innerHTML = `
            <h3>${t("settings.java.location", { n: major })}</h3>
            <div class="java-path-row">
              <input data-java-path="${major}" value="" placeholder="/path/to/java" />
            </div>
            <div class="java-actions">
              <button type="button" class="btn btn-ghost" data-java-install="${major}">${t("settings.java.installRec")}</button>
              <button type="button" class="btn btn-ghost" data-java-detect="${major}">${t("settings.java.detect")}</button>
              <button type="button" class="btn btn-ghost" data-java-browse="${major}">${t("settings.java.browse")}</button>
            </div>
          `;
          const input = section.querySelector<HTMLInputElement>("input")!;
          input.value = paths[major] ?? "";
          input.addEventListener("change", () => {
            void (async () => {
              const path = input.value.trim();
              paths[major] = path;
              if (!inTauri) return;
              try {
                launcherCfg = await invoke<LauncherConfig>("set_java_version", {
                  major,
                  path: path || null,
                });
                setStatus(status, t("instance.settings.saved"), "ok");
              } catch (e) {
                setStatus(status, String(e), "err");
              }
            })();
          });
          section.querySelector(`[data-java-detect="${major}"]`)?.addEventListener("click", () => {
            openJavaDetectModal(major, paths[major] ?? "", (path) => {
              input.value = path;
              paths[major] = path;
              void (async () => {
                if (!inTauri) return;
                launcherCfg = await invoke<LauncherConfig>("set_java_version", { major, path });
                setStatus(status, t("instance.settings.saved"), "ok");
              })();
            });
          });
          section.querySelector(`[data-java-browse="${major}"]`)?.addEventListener("click", () => {
            void (async () => {
              const picked = await openDialog({ multiple: false });
              if (typeof picked !== "string") return;
              input.value = picked;
              paths[major] = picked;
              if (!inTauri) return;
              launcherCfg = await invoke<LauncherConfig>("set_java_version", {
                major,
                path: picked,
              });
              setStatus(status, t("instance.settings.saved"), "ok");
            })();
          });
          section.querySelector(`[data-java-install="${major}"]`)?.addEventListener("click", () => {
            void (async () => {
              if (!inTauri) {
                setStatus(status, t("play.browser"), "muted");
                return;
              }
              setStatus(status, t("settings.java.installing", { n: major }), "muted");
              try {
                const installed = await invoke<JavaInstallation>("install_java", { major });
                input.value = installed.path;
                paths[major] = installed.path;
                launcherCfg = await invoke<LauncherConfig>("get_config");
                setStatus(status, t("settings.java.installed", { n: major }), "ok");
              } catch (e) {
                setStatus(status, friendlyError(e), "err");
              }
            })();
          });
          box.append(section);
        }
      };
      renderMajors();
    } else if (settingsTab === "localization") {
      pane.innerHTML = `
        <h2>${t("settings.localization")}</h2>
        <p class="muted">${t("settings.locale.desc")}</p>
        <div class="chips" id="locale-chips"></div>
        <p id="settings-status" class="status"></p>
      `;
      const box = pane.querySelector<HTMLElement>("#locale-chips")!;
      const status = pane.querySelector<HTMLElement>("#settings-status")!;
      for (const loc of ["ru", "en"] as const) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = `chip${getLocale() === loc ? " is-selected" : ""}`;
        btn.textContent = (getLocale() === loc ? "✓ " : "") + t(`settings.locale.${loc}`);
        btn.addEventListener("click", () => {
          void (async () => {
            setLocale(loc);
            localStorage.setItem(STORAGE_LOCALE, loc);
            if (inTauri) {
              launcherCfg = await invoke<LauncherConfig>("set_locale", { locale: loc });
            }
            applyChromeI18n();
            // Re-paint content under the modal (settings is not a content-route).
            paintCurrentRoute();
            settingsTab = "localization";
            setRailHighlight("settings");
            await openAppSettingsModal();
            setStatus(
              modalAppSettings.querySelector<HTMLElement>("#settings-status") ?? status,
              "OK",
              "ok",
            );
          })();
        });
        box.append(btn);
      }
    } else {
      const channel = launcherCfg?.updater?.channel ?? "stable";
      pane.innerHTML = `
        <h2>${t("settings.updates")}</h2>
        <p class="muted">${t("settings.updates.desc")}</p>
        <p class="hint">${t("settings.updates.channel")}: <code>${channel}</code></p>
        <button type="button" class="btn btn-ghost" id="btn-check-updates">${t("settings.updates.check")}</button>
        <p id="settings-status" class="status"></p>
      `;
      const status = pane.querySelector<HTMLElement>("#settings-status")!;
      pane.querySelector("#btn-check-updates")?.addEventListener("click", () => {
        void (async () => {
          try {
            if (inTauri) {
              launcherCfg = await invoke<LauncherConfig>("check_updates");
            }
            setStatus(status, t("settings.updates.stub", { channel }), "muted");
          } catch (e) {
            setStatus(status, String(e), "err");
          }
        })();
      });
    }
  };

  await paint();
}

async function onPlayClick(inst: Instance) {
  if (!ensureCanPlay()) return;
  if (inTauri) {
    try {
      const running = await invoke<boolean>("is_pack_running", { packId: inst.id });
      if (running) {
        await invoke("stop_pack", { packId: inst.id });
        setTitlebarIdle(t("play.stopped"));
        await refreshPlayButton(inst);
        return;
      }
    } catch {
      /* ignore */
    }
  }
  if (inst.installStatus === "installing" && activeInstallId === inst.id) {
    setTitlebarIdle(t("instance.waitInstall"));
    return;
  }
  inst.lastPlayedAt = Date.now();
  saveInstances();
  renderRecent();
  void tryLaunch(inst);
}

async function refreshPlayButton(inst: Instance) {
  const btn = document.querySelector<HTMLButtonElement>("#btn-play");
  if (!btn) return;
  btn.classList.remove("btn-stop", "is-launching");
  if (inst.installStatus === "installing" && activeInstallId === inst.id) {
    btn.disabled = true;
    btn.textContent = `… ${t("instance.installing")}`;
    stopRunningPoll();
    return;
  }
  btn.disabled = false;
  if (inTauri) {
    try {
      const running = await invoke<boolean>("is_pack_running", { packId: inst.id });
      if (running) {
        btn.classList.add("btn-stop");
        btn.textContent = `■ ${t("play.stop")}`;
        startRunningPoll(inst);
        return;
      }
    } catch {
      /* ignore */
    }
  }
  stopRunningPoll();
  btn.textContent = `▶ ${t("instance.play")}`;
}

function shortProgressFile(file: string, max = 40): string {
  if (file.length <= max) return file;
  return `…${file.slice(-(max - 1))}`;
}

function progressPct(p: InstallProgress): number {
  if (
    p.fileBytesDone != null &&
    p.fileBytesTotal != null &&
    p.fileBytesTotal > 0 &&
    Number.isFinite(p.fileBytesDone) &&
    Number.isFinite(p.fileBytesTotal)
  ) {
    const fileFrac = Math.min(1, Math.max(0, p.fileBytesDone / p.fileBytesTotal));
    if (p.totalFiles > 0) {
      // Blend file index with in-file bytes so large jars (client.jar) don't look stuck at 0/N.
      return Math.min(99, ((p.doneFiles + fileFrac) / p.totalFiles) * 100);
    }
    return Math.min(99, fileFrac * 100);
  }
  if (p.totalFiles > 0) {
    return Math.min(100, (p.doneFiles / p.totalFiles) * 100);
  }
  return 8;
}

function updatePlayProgress(p: InstallProgress) {
  updateDownloadTip(p);
  if (p.packId === "__meta_prefetch__") {
    metaPrefetch = {
      ready: p.phase === "done",
      message: p.message,
      doneFiles: p.doneFiles,
      totalFiles: p.totalFiles,
      minecraft: metaPrefetch.minecraft,
    };
    if (p.phase !== "done" && !activeInstallId) {
      // Keep prefetch visually distinct from instance install (create must not look like install).
      const file = p.currentFile ? shortProgressFile(p.currentFile) : "";
      const counts = p.totalFiles > 0 ? ` ${p.doneFiles}/${p.totalFiles}` : "";
      const detail = file ? ` · ${file}${counts}` : counts;
      setTitlebarBusy(`${t("meta.prefetching")}${detail}`, progressPct(p));
    }
    if (p.phase === "done") {
      metaPrefetch.ready = true;
      if (!activeInstallId) clearTitlebarBusy(t("status.ready"));
    }
    return;
  }
  // Global install progress — keep showing even when user navigates away.
  const file = p.currentFile || p.message;
  const short = shortProgressFile(file, 36);
  const counts = p.totalFiles > 0 ? ` ${p.doneFiles}/${p.totalFiles}` : "";
  if (p.phase === "done") {
    clearTitlebarBusy(p.message || t("status.ready"));
  } else {
    setTitlebarBusy(`${t("instance.installing")} ${short}${counts}`, progressPct(p));
  }
}

function ensureMetaReady(inst?: Instance): boolean {
  if (metaPrefetch.ready || !inTauri) return true;
  if (inst?.installStatus === "ready") return true;
  if (metaPrefetchFailed) {
    setTitlebarIdle(t("meta.failedContinue"));
    return true;
  }
  setTitlebarBusy(t("meta.wait"), metaPrefetch.totalFiles > 0
    ? Math.min(100, (metaPrefetch.doneFiles / metaPrefetch.totalFiles) * 100)
    : 8);
  return false;
}

function launchOverrides(inst: Instance) {
  const extraJvm = inst.settings.overrideJvmArgs
    ? inst.settings.jvmArgs
        .split(/\s+/)
        .map((s) => s.trim())
        .filter(Boolean)
    : [];
  const envVars = inst.settings.overrideEnvVars
    ? inst.settings.envVars
        .trim()
        .split(/\s+/)
        .filter(Boolean)
        .map((pair) => {
          const i = pair.indexOf("=");
          if (i <= 0) return null;
          return [pair.slice(0, i), pair.slice(i + 1)] as [string, string];
        })
        .filter((x): x is [string, string] => Boolean(x))
    : [];
  return {
    memoryMb: inst.settings.overrideMemory ? inst.settings.memoryMb : undefined,
    width: inst.settings.width,
    height: inst.settings.height,
    jvmArgs: extraJvm,
    javaPath: inst.settings.overrideJavaPath ? inst.settings.javaPath || undefined : undefined,
    envVars: envVars.length ? envVars : undefined,
    preLaunch: inst.settings.overrideHooks ? inst.settings.preLaunch || undefined : undefined,
    wrapper: inst.settings.overrideHooks ? inst.settings.wrapper || undefined : undefined,
    postExit: inst.settings.overrideHooks ? inst.settings.postExit || undefined : undefined,
    serverAddress: inst.serverAddress || undefined,
    serverPort: inst.serverPort || undefined,
  };
}

async function tryLaunch(inst: Instance) {
  if (!ensureCanPlay()) return;
  if (!inTauri) {
    setTitlebarMessage(t("play.browser"));
    return;
  }
  if (!ensureMetaReady(inst)) return;
  const btn = document.querySelector<HTMLButtonElement>("#btn-play");
  // Modrinth-like: Play installs when needed, then launches — never dead-end on "install first".
  if (inst.installStatus !== "ready") {
    await runInstall(inst);
  }
  if (inst.installStatus !== "ready") return;

  const invokeLaunch = () =>
    invoke<{ message: string; pid: number }>("launch_pack", {
      packId: inst.id,
      overrides: launchOverrides(inst),
    });

  try {
    if (btn) {
      btn.disabled = true;
      btn.classList.add("is-launching");
      btn.textContent = `… ${t("play.launching")}`;
    }
    setTitlebarMessage(t("play.launching"));
    let res: { message: string; pid: number };
    try {
      res = await invokeLaunch();
    } catch (e) {
      // Stale "ready" marker / missing shared jars — repair once then retry launch.
      if (!/Pack is not ready/i.test(String(e))) throw e;
      setTitlebarBusy(t("instance.installing"), 5);
      await runInstall(inst);
      if (inst.installStatus !== "ready") {
        // Button was already set to launching; restore Play CTA on failed repair.
        await refreshPlayButton(inst);
        return;
      }
      if (btn) {
        btn.disabled = true;
        btn.classList.add("is-launching");
        btn.textContent = `… ${t("play.launching")}`;
      }
      setTitlebarMessage(t("play.launching"));
      res = await invokeLaunch();
    }
    clearTitlebarBusy(res.message || `PID ${res.pid}`);
    await refreshPlayButton(inst);
  } catch (e) {
    titlebarBusy = false;
    hideTopProgress();
    setTitlebarMessage(friendlyError(e));
    await refreshPlayButton(inst);
  }
}

async function runInstall(inst: Instance) {
  if (!inTauri) {
    inst.installStatus = "ready";
    inst.installMessage = t("play.browser");
    saveInstances();
    return;
  }
  if (!ensureMetaReady(inst)) return;
  activeInstallId = inst.id;
  inst.installStatus = "installing";
  inst.installMessage = t("instance.installing");
  saveInstances();
  setTitlebarBusy(t("instance.installing"), 5);
  if (document.querySelector(".instance-page") && selectedInstanceId === inst.id) {
    renderInstance();
  }
  try {
    const status = await invoke<{ ready: boolean; version?: string }>("install_instance", {
      id: inst.id,
      name: inst.name,
      loader: inst.loader,
      minecraft: inst.minecraft,
      loaderVersion: inst.loaderVersion,
    });
    inst.installStatus = status.ready ? "ready" : "error";
    inst.installMessage = status.ready
      ? `${t("instance.installDone")}${status.version ? ` · ${status.version}` : ""}`
      : t("instance.installFail");
    saveInstances();
    await hydrateInstancesFromDisk();
    clearTitlebarBusy(inst.installMessage);
    if (selectedInstanceId === inst.id && document.querySelector(".instance-page")) {
      renderInstance();
    }
  } catch (e) {
    inst.installStatus = "error";
    inst.installMessage = friendlyError(e);
    saveInstances();
    setTitlebarError(inst.installMessage);
    if (document.querySelector(".instance-page") && selectedInstanceId === inst.id) {
      renderInstance();
    }
  } finally {
    if (activeInstallId === inst.id) activeInstallId = null;
  }
}

function suggestedInstanceName(): string {
  const minecraft = mcCombo.getValue() || readySpec?.minecraft || "";
  const loader = selectedLoader || readySpec?.loader || "vanilla";
  if (!minecraft) return prettyLoader(loader);
  if (loader === "vanilla") return `Vanilla ${minecraft}`;
  return `${prettyLoader(loader)} ${minecraft}`;
}

function refreshCreateNameHint() {
  instName.placeholder = suggestedInstanceName();
}

function paintCreateIconPreview() {
  createIconPreview.replaceChildren();
  createIconPreview.classList.toggle("has-image", Boolean(createIconDataUrl));
  if (createIconDataUrl) {
    const img = document.createElement("img");
    img.src = createIconDataUrl;
    img.alt = "";
    createIconPreview.append(img);
  } else {
    createIconPreview.textContent = (instName.value.trim() || suggestedInstanceName() || "?").slice(0, 1).toUpperCase();
  }
  btnCreateIconClear.disabled = !createIconPath;
}

async function pickCreateIcon() {
  if (!inTauri) {
    setStatus(createStatus, t("play.browser"), "muted");
    return;
  }
  const picked = await openDialog({
    multiple: false,
    filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }],
  });
  const path = typeof picked === "string" ? picked : Array.isArray(picked) ? picked[0] : null;
  if (!path) return;
  try {
    const dataUrl = await invoke<string>("read_image_data_url", { path });
    createIconPath = path;
    createIconDataUrl = dataUrl;
    paintCreateIconPreview();
  } catch (e) {
    setStatus(createStatus, friendlyError(e), "err");
  }
}

function clearCreateIcon() {
  createIconPath = null;
  createIconDataUrl = null;
  paintCreateIconPreview();
}

async function createInstance() {
  // Prefer live select values; fall back to readySpec when still syncing.
  const minecraft = mcCombo.getValue() || readySpec?.minecraft || "";
  const loader = selectedLoader || readySpec?.loader || "vanilla";
  const loaderVersion =
    loader === "vanilla"
      ? ""
      : loaderCombo.getValue() || readySpec?.loaderVersion || "";

  if (!minecraft) {
    setStatus(createStatus, t("create.needMc"), "err");
    return;
  }
  if (loader !== "vanilla" && !loaderVersion) {
    setStatus(createStatus, t("create.needLoader"), "err");
    return;
  }

  const name = instName.value.trim() || suggestedInstanceName();
  setStatus(createStatus, t("create.preparing"), "muted");

  try {
    let disk: DiskInstance | null = null;
    if (inTauri) {
      disk = await invoke<DiskInstance>("prepare_instance", {
        name,
        loader,
        minecraft,
        loaderVersion,
        iconPath: createIconPath,
      });
    }
    const id =
      disk?.id ??
      `inst_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 7)}`;
    const inst = normalizeInstance({
      id,
      name: (disk?.name || name).slice(0, 48),
      loader: disk?.loader || loader,
      minecraft: disk?.minecraft || minecraft,
      loaderVersion: disk?.loaderVersion || loaderVersion,
      createdAt: disk ? createdAtMs(disk.createdAt) : Date.now(),
      content: [],
      worlds: [],
      settings: DEFAULT_SETTINGS(),
      installStatus: "pending",
      installMessage: undefined,
      hasIcon: disk?.hasIcon ?? Boolean(createIconDataUrl),
      iconUrl: createIconDataUrl ?? (disk?.hasIcon ? undefined : null),
    });
    instances = [inst, ...instances.filter((i) => i.id !== id)];
    saveInstances();
    selectedInstanceId = id;
    instanceTab = "content";
    closeCreateModal();
    routeTo("instance", id);
  } catch (e) {
    setStatus(createStatus, friendlyError(e), "err");
  }
}

const CREATE_STEP_KEYS = [
  "",
  "create.step.loader",
  "create.step.mc",
  "create.step.loaderVer",
  "create.step.name",
  "create.step.icon",
] as const;

function createMaxStep(): number {
  return 5;
}

function syncCreateWizardStep() {
  const max = createMaxStep();
  if (createWizardStep < 1) createWizardStep = 1;
  if (createWizardStep > max) createWizardStep = max;
  for (let i = 1; i <= 5; i++) {
    const panel = modalCreate.querySelector<HTMLElement>(`#create-step-${i}`);
    if (panel) panel.hidden = createWizardStep !== i;
  }
  modalCreate.querySelectorAll<HTMLElement>("[data-cstep]").forEach((el) => {
    const n = Number(el.dataset.cstep);
    el.classList.toggle("is-active", n === createWizardStep);
    el.classList.toggle("is-done", n < createWizardStep);
  });
  const label = modalCreate.querySelector<HTMLElement>("#create-step-label");
  if (label) label.textContent = t(CREATE_STEP_KEYS[createWizardStep] || "create.title");
  const vanillaSkip = modalCreate.querySelector<HTMLElement>("#create-vanilla-skip");
  if (vanillaSkip) vanillaSkip.hidden = !(createWizardStep === 3 && selectedLoader === "vanilla");
  const loaderField = modalCreate.querySelector<HTMLElement>("#loader-version-field");
  if (loaderField) loaderField.hidden = selectedLoader === "vanilla";
  const nextBtn = modalCreate.querySelector<HTMLButtonElement>("#btn-create-next")!;
  const submitBtn = modalCreate.querySelector<HTMLButtonElement>("#btn-create")!;
  const backBtn = modalCreate.querySelector<HTMLButtonElement>("#btn-create-back")!;
  backBtn.hidden = createWizardStep <= 1;
  const last = createWizardStep >= max;
  nextBtn.hidden = last;
  submitBtn.hidden = !last;
  if (!last) {
    nextBtn.disabled = !canAdvanceCreateStep();
  } else {
    submitBtn.disabled = !canAdvanceCreateStep();
  }
}

function canAdvanceCreateStep(): boolean {
  if (createWizardStep === 1) return Boolean(selectedLoader);
  if (createWizardStep === 2) return Boolean(mcCombo.getValue());
  if (createWizardStep === 3) {
    if (selectedLoader === "vanilla") return true;
    return Boolean(loaderCombo.getValue());
  }
  if (createWizardStep === 4) return Boolean(instName.value.trim());
  if (createWizardStep === 5) return true; // icon optional
  return false;
}

function advanceCreateStep() {
  if (!canAdvanceCreateStep()) return;
  if (createWizardStep === 1) {
    beginCreateRefresh();
    void refreshMinecraftVersions().then(() => {
      syncCreateWizardStep();
    });
  }
  if (createWizardStep === 2) {
    void refreshLoaderVersions().then(() => syncCreateWizardStep());
  }
  if (createWizardStep >= createMaxStep()) return;
  createWizardStep += 1;
  // Skip loader-version step content is still shown with vanilla skip hint
  syncCreateWizardStep();
  if (createWizardStep === 4) {
    refreshCreateNameHint();
    instName.focus();
  }
  if (createWizardStep === 5) paintCreateIconPreview();
}

function retreatCreateStep() {
  if (createWizardStep <= 1) return;
  createWizardStep -= 1;
  syncCreateWizardStep();
}

function openCreateModal() {
  createWizardStep = 1;
  showModal(modalCreate, "#loader-chips");
  instName.value = "";
  selectedLoader = "vanilla";
  createShowSnapshots = false;
  clearCreateIcon();
  mcCombo.close();
  loaderCombo.close();
  mcCombo.setLabels({
    showAll: t("create.showAllVersions"),
    hideAll: t("create.hideSnapshots"),
    placeholder: t("create.searchVersion"),
    toggle: t("create.toggleVersions"),
    empty: t("create.noVersions"),
  });
  loaderCombo.setLabels({
    placeholder: t("create.searchVersion"),
    toggle: t("create.toggleVersions"),
    empty: t("create.noVersions"),
  });
  beginCreateRefresh();
  renderChips();
  refreshCreateNameHint();
  paintCreateIconPreview();
  syncCreateWizardStep();
  void refreshMinecraftVersions().then(() => {
    refreshCreateNameHint();
    paintCreateIconPreview();
    syncCreateWizardStep();
  });
}

function closeCreateModal() {
  mcCombo.close();
  loaderCombo.close();
  hideModal(modalCreate);
  clearCreateIcon();
}

function renderChips() {
  chips.replaceChildren();
  for (const loader of LOADERS) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `chip${loader.id === selectedLoader ? " is-selected" : ""}`;
    btn.textContent = (loader.id === selectedLoader ? "✓ " : "") + loader.name;
    btn.addEventListener("click", () => {
      selectedLoader = loader.id;
      renderChips();
      refreshCreateNameHint();
      syncCreateWizardStep();
      void refreshMinecraftVersions().then(() => syncCreateWizardStep());
    });
    chips.append(btn);
  }
}

async function tauriOrBrowserMcVersions(loader: string, includeSnapshots: boolean) {
  if (inTauri) {
    const rows = await invoke<Array<{ id: string }>>("list_minecraft_versions", {
      loader,
      includeSnapshots,
    });
    return rows.map((r) => r.id);
  }
  if (loader === "fabric") {
    const res = await fetch("/proxy/fabric/v2/versions/game");
    const data = (await res.json()) as Array<{ version: string; stable: boolean }>;
    return data.filter((v) => includeSnapshots || v.stable).map((v) => v.version);
  }
  const res = await fetch("/proxy/mojang/mc/game/version_manifest_v2.json");
  const data = (await res.json()) as { versions: Array<{ id: string; type: string }> };
  return data.versions
    .filter((v) => includeSnapshots || v.type === "release")
    .map((v) => v.id);
}

async function tauriOrBrowserLoaderVersions(loader: string, minecraft: string) {
  if (loader === "vanilla") return [] as string[];
  if (inTauri) return invoke<string[]>("list_loader_versions", { loader, minecraft });
  if (loader === "fabric") {
    const res = await fetch(`/proxy/fabric/v2/versions/loader/${encodeURIComponent(minecraft)}`);
    if (!res.ok) return [];
    const data = (await res.json()) as Array<{ loader: { version: string } }>;
    return data.map((r) => r.loader.version);
  }
  if (loader === "quilt") {
    const res = await fetch(`/proxy/quilt/v3/versions/loader/${encodeURIComponent(minecraft)}`);
    if (!res.ok) return [];
    const data = (await res.json()) as Array<{ loader: { version: string } }>;
    return data.map((r) => r.loader.version);
  }
  if (loader === "forge") {
    const res = await fetch("/proxy/forge/net/minecraftforge/forge/maven-metadata.xml");
    const xml = await res.text();
    const prefix = `${minecraft}-`;
    return [...xml.matchAll(/<version>([^<]+)<\/version>/g)]
      .map((m) => m[1])
      .filter((v) => v.startsWith(prefix))
      .map((v) => v.slice(prefix.length))
      .reverse();
  }
  if (loader === "neoforge") {
    const res = await fetch("/proxy/neoforge/releases/net/neoforged/neoforge/maven-metadata.xml");
    const xml = await res.text();
    const parts = minecraft.split(".");
    const filter = parts.length >= 2 ? `${parts[1]}.${parts[2] ?? "0"}` : minecraft;
    return [...xml.matchAll(/<version>([^<]+)<\/version>/g)]
      .map((m) => m[1])
      .filter(
        (v) =>
          v === filter ||
          v.startsWith(`${filter}.`) ||
          v.startsWith(`${filter}-`),
      )
      .reverse();
  }
  return [];
}

async function refreshMinecraftVersions() {
  const epoch = beginCreateRefresh();
  const requestId = ++mcRequestId;
  ++loaderRequestId; // drop any in-flight loader apply
  setStatus(createStatus, t("create.loadingVersions"), "muted");
  const needsLoader = selectedLoader !== "vanilla";
  loaderField.hidden = !needsLoader;
  loaderLabel.textContent = t("create.loaderVersion", { loader: prettyLoader(selectedLoader) });
  try {
    const ids = await tauriOrBrowserMcVersions(selectedLoader, createShowSnapshots);
    if (requestId !== mcRequestId || epoch !== createRefreshEpoch) return;
    const preferred = ids.find((id) => id === "1.20.1") ?? ids[0] ?? "";
    mcCombo.setOptions(ids, preferred);
    await refreshLoaderVersions({ nested: true, epoch });
    if (requestId !== mcRequestId || epoch !== createRefreshEpoch) return;
    const ready =
      Boolean(mcCombo.getValue()) &&
      (selectedLoader === "vanilla" || Boolean(loaderCombo.getValue()));
    setCreateReady(epoch, ready);
    refreshCreateNameHint();
    setStatus(
      createStatus,
      ids.length ? t("create.mcCount", { n: ids.length }) : t("create.noVersions"),
      ids.length ? "ok" : "err",
    );
  } catch (e) {
    if (requestId !== mcRequestId || epoch !== createRefreshEpoch) return;
    mcCombo.setOptions([]);
    loaderCombo.setOptions([]);
    setCreateReady(epoch, false);
    setStatus(createStatus, String(e), "err");
  }
}

async function refreshLoaderVersions(opts?: { nested?: boolean; epoch?: number }) {
  const epoch = opts?.nested ? opts.epoch! : beginCreateRefresh();
  const requestId = ++loaderRequestId;
  if (selectedLoader === "vanilla") {
    loaderCombo.setOptions([]);
    if (!opts?.nested) setCreateReady(epoch, Boolean(mcCombo.getValue()));
    return;
  }
  const mc = mcCombo.getValue();
  if (!mc) {
    loaderCombo.setOptions([]);
    if (!opts?.nested) setCreateReady(epoch, false);
    return;
  }
  try {
    const loaders = await tauriOrBrowserLoaderVersions(selectedLoader, mc);
    if (requestId !== loaderRequestId || epoch !== createRefreshEpoch) return;
    loaderCombo.setOptions(loaders, loaders[0]);
    if (!opts?.nested) {
      setCreateReady(epoch, Boolean(mcCombo.getValue() && loaderCombo.getValue()));
    }
  } catch (e) {
    if (requestId !== loaderRequestId || epoch !== createRefreshEpoch) return;
    loaderCombo.setOptions([]);
    if (!opts?.nested) setCreateReady(epoch, false);
    setStatus(createStatus, String(e), "err");
  }
}

root.querySelectorAll<HTMLButtonElement>(".rail-btn[data-route]").forEach((btn) => {
  btn.addEventListener("click", () => {
    if (isProfileGateOpen()) return;
    const r = btn.dataset.route!;
    if (r === "browse") {
      browseInstanceId = null;
      selectedProjectHit = null;
      routeTo("browse");
      return;
    }
    routeTo(r);
  });
});
root.querySelectorAll<HTMLElement>("[data-close='inst-settings']").forEach((el) => {
  el.addEventListener("click", () => {
    hideModal(modalInstSettings);
  });
});
root.querySelectorAll<HTMLElement>("[data-close='app-settings']").forEach((el) => {
  el.addEventListener("click", () => {
    closeAppSettingsOverlay();
  });
});
root.querySelectorAll<HTMLElement>("[data-close='java-detect']").forEach((el) => {
  el.addEventListener("click", () => {
    hideModal(modalJavaDetect);
    javaDetectCallback = null;
  });
});
modalInstSettings.addEventListener("click", (e) => {
  if (e.target === modalInstSettings) hideModal(modalInstSettings);
});
modalAppSettings.addEventListener("click", (e) => {
  if (e.target === modalAppSettings) closeAppSettingsOverlay();
});
modalJavaDetect.addEventListener("click", (e) => {
  if (e.target === modalJavaDetect) {
    hideModal(modalJavaDetect);
    javaDetectCallback = null;
  }
});
root.addEventListener("click", (e) => {
  const el = (e.target as HTMLElement | null)?.closest?.("[data-action]") as HTMLElement | null;
  if (!el || !root.contains(el)) return;
  const action = el.getAttribute("data-action");
  if (action === "create" || action === "create-empty") {
    e.preventDefault();
    if (isProfileGateOpen()) return;
    openCreateModal();
  } else if (action === "accounts") {
    e.preventDefault();
    if (isProfileGateOpen()) return;
    openProfileWizard({ allowClose: true });
  }
});
root.querySelectorAll<HTMLElement>("[data-close='create']").forEach((el) => {
  el.addEventListener("click", closeCreateModal);
});
root.querySelector("#btn-create-next")?.addEventListener("click", () => {
  advanceCreateStep();
});
root.querySelector("#btn-create-back")?.addEventListener("click", () => {
  retreatCreateStep();
});
root.querySelector("#create-type-modpack")?.addEventListener("click", () => {
  closeCreateModal();
  browseProjectType = "modpack";
  browseInstanceId = null;
  routeTo("browse");
});
root.querySelector("#create-import-mrpack")?.addEventListener("click", () => {
  void (async () => {
    if (!inTauri) {
      setTitlebarMessage(t("create.importMrpackBrowser"));
      return;
    }
    try {
      const selected = await openDialog({
        multiple: false,
        title: t("create.type.importMrpack"),
        filters: [{ name: "mrpack", extensions: ["mrpack", "zip"] }],
      });
      if (typeof selected !== "string") return;
      closeCreateModal();
      setTitlebarMessage(t("create.importMrpackWorking"));
      const result = await invoke<{
        instance: DiskInstance;
        message: string;
        partial: boolean;
      }>("import_mrpack", { path: selected });
      await hydrateInstancesFromDisk();
      setTitlebarMessage(result.message);
      selectedInstanceId = result.instance.id;
      routeTo("instance", result.instance.id);
    } catch (e) {
      setTitlebarError(friendlyError(e));
    }
  })();
});
histBack.addEventListener("click", goBack);
histForward.addEventListener("click", goForward);
root.querySelector("#btn-wizard")?.addEventListener("click", () => void finishWizard());
root.querySelector("#btn-wizard-auth")?.addEventListener("click", () => void onMicrosoftAuthButtonClick());
modalWizard.addEventListener("click", (e) => {
  if (e.target === modalWizard && wizardAllowClose) hideModal(modalWizard);
});
btnCreateIcon.addEventListener("click", () => void pickCreateIcon());
btnCreateIconClear.addEventListener("click", () => clearCreateIcon());
instName.addEventListener("input", () => {
  if (!createIconDataUrl) paintCreateIconPreview();
  syncCreateWizardStep();
});
btnCreate.addEventListener("click", () => void createInstance());
modalCreate.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || event.shiftKey) return;
  if (event.target instanceof HTMLTextAreaElement) return;
  event.preventDefault();
  if (createWizardStep < createMaxStep()) {
    advanceCreateStep();
    return;
  }
  if (btnCreate.disabled) return;
  void createInstance();
});
modalCreate.addEventListener("click", (e) => {
  if (e.target === modalCreate) closeCreateModal();
});

if (inTauri) {
  const win = getCurrentWindow();
  const bindWin = (minId: string, maxId: string, closeId: string) => {
    root.querySelector(minId)?.addEventListener("click", () => void win.minimize());
    root.querySelector(maxId)?.addEventListener("click", () => void win.toggleMaximize());
    root.querySelector(closeId)?.addEventListener("click", () => void win.close());
  };
  bindWin("#win-min", "#win-max", "#win-close");
  bindWin("#splash-min", "#splash-max", "#splash-close");

  void listen<InstallProgress>("install://progress", (event) => {
    const p = event.payload;
    updatePlayProgress(p);
    if (p.packId === "__meta_prefetch__") return;
    const inst = instances.find((i) => i.id === p.packId);
    if (!inst) return;
    if (inst.installStatus === "ready" || inst.installStatus === "error") return;
    if (p.phase === "done") {
      inst.installMessage = p.message;
      saveInstances();
      return;
    }
    const progress = p.totalFiles > 0 ? ` ${p.doneFiles}/${p.totalFiles}` : "";
    inst.installStatus = "installing";
    inst.installMessage = `${p.message}${progress}`;
    saveInstances();
    if (selectedInstanceId === inst.id) {
      void refreshPlayButton(inst);
    }
  });

  void listen<{ packId: string; pid: number }>("process://finished", (event) => {
    const packId = event.payload.packId;
    // Ignore exits for other instances so we don't stop B's poll / idle titlebar.
    if (selectedInstanceId !== packId) return;
    const inst = instances.find((i) => i.id === packId);
    stopRunningPoll();
    setTitlebarIdle(t("play.exited"));
    if (inst) void refreshPlayButton(inst);
  });
}

void boot();
