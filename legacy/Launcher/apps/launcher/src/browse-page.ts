/**
 * Dedicated Browse / Project screens (Modrinth App parity).
 * Content tab CTA navigates here — no inline Modrinth search as primary UX.
 *
 * Patterns from modrinth/code:
 * - packages/ui/.../browse-tab/layout.vue + sidebar.vue
 * - apps/app-frontend/src/pages/Browse.vue
 * - apps/app-frontend/src/pages/project/Index.vue
 */

import { invoke } from "@tauri-apps/api/core";

export type BrowseProjectType = "mod" | "modpack" | "resourcepack" | "datapack" | "shader";

export type BrowseHit = {
  projectId: string;
  slug: string;
  title: string;
  description: string;
  projectType: string;
  downloads: number;
  iconUrl?: string | null;
  categories?: string[];
};

export type BrowseInstanceCtx = {
  id: string;
  name: string;
  loader: string;
  minecraft: string;
};

export type BrowseDeps = {
  t: (key: string, vars?: Record<string, string | number>) => string;
  inTauri: boolean;
  isSafeModrinthMediaUrl: (raw: string) => boolean;
  friendlyError: (e: unknown) => string;
  prettyLoader: (id: string) => string;
  onOpenProject: (hit: BrowseHit, projectType: BrowseProjectType) => void;
  onBackToContent?: () => void;
  setStatus: (msg: string) => void;
  getInstalledProjectIds?: () => Promise<Set<string>>;
  /** After importing a modpack as a new instance (global Discover). */
  onModpackImported?: (instanceId: string) => void | Promise<void>;
};

const PAGE_SIZES = [10, 20, 50] as const;
const SORTS = [
  { id: "relevance", key: "browse.sort.relevance" },
  { id: "downloads", key: "browse.sort.downloads" },
  { id: "follows", key: "browse.sort.follows" },
  { id: "newest", key: "browse.sort.newest" },
  { id: "updated", key: "browse.sort.updated" },
] as const;

const TYPE_TABS: Array<{ id: BrowseProjectType; key: string }> = [
  { id: "mod", key: "browse.type.mods" },
  { id: "modpack", key: "browse.type.modpacks" },
  { id: "resourcepack", key: "browse.type.resourcepacks" },
  { id: "datapack", key: "browse.type.datapacks" },
  { id: "shader", key: "browse.type.shaders" },
];

const MOD_CATEGORIES = [
  "",
  "optimization",
  "technology",
  "adventure",
  "magic",
  "utility",
  "decoration",
  "library",
  "worldgen",
  "food",
  "equipment",
  "storage",
  "transportation",
];

function formatDownloads(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

export function mountBrowsePage(
  root: HTMLElement,
  opts: {
    deps: BrowseDeps;
    instance: BrowseInstanceCtx | null;
    projectType: BrowseProjectType;
    onProjectTypeChange: (t: BrowseProjectType) => void;
  },
): void {
  const { deps, instance } = opts;
  let projectType = opts.projectType;
  let query = "";
  let sort = "relevance";
  let pageSize: (typeof PAGE_SIZES)[number] = 20;
  let offset = 0;
  let total = 0;
  let category = "";
  let hideInstalled = true;
  let filtersLocked = Boolean(instance);
  let optionalDeps = false;
  let busy = false;
  let searchGen = 0;
  let installedIds = new Set<string>();
  let searchTimer: number | undefined;

  const loaderLocked = filtersLocked && instance ? instance.loader : "";
  const mcLocked = filtersLocked && instance ? instance.minecraft : "";

  root.innerHTML = `
    <section class="page browse-page">
      ${
        instance
          ? `<header class="browse-install-header">
        <button type="button" class="btn btn-ghost sm" id="browse-back">${deps.t("browse.back")}</button>
        <div class="browse-install-meta">
          <strong id="browse-inst-name"></strong>
          <span class="muted" id="browse-inst-meta"></span>
        </div>
      </header>`
          : `<header class="page-head">
        <h1>${deps.t("browse.discover")}</h1>
        <p class="muted">${deps.t("browse.discoverHint")}</p>
      </header>`
      }
      <div class="browse-type-tabs" id="browse-type-tabs" role="tablist"></div>
      <div class="browse-layout">
        <div class="browse-main">
          <div class="toolbar browse-toolbar">
            <input class="search browse-search" id="browse-query" placeholder="" />
            <label class="tiny muted browse-sort-label">${deps.t("browse.sortLabel")}
              <select id="browse-sort" class="search"></select>
            </label>
            <label class="tiny muted">${deps.t("browse.pageSize")}
              <select id="browse-pagesize" class="search"></select>
            </label>
            <label class="check-row tiny">
              <input id="browse-optional-deps" type="checkbox" />
              <span>${deps.t("instance.browseMods.optionalDeps")}</span>
            </label>
          </div>
          <div id="browse-results" class="browse-results muted">${deps.t("browse.empty")}</div>
          <div class="browse-pager" id="browse-pager" hidden>
            <button type="button" class="btn btn-ghost sm" id="browse-prev">${deps.t("browse.prev")}</button>
            <span class="muted" id="browse-page-label"></span>
            <button type="button" class="btn btn-ghost sm" id="browse-next">${deps.t("browse.next")}</button>
          </div>
        </div>
        <aside class="browse-sidebar" aria-label="${deps.t("browse.filters")}">
          <label class="check-row">
            <input type="checkbox" id="browse-hide-installed" checked />
            <span>${deps.t("browse.hideInstalled")}</span>
          </label>
          <div class="browse-locked" id="browse-locked">
            <div class="browse-locked-row">
              <span class="muted">${deps.t("browse.gameVersion")}</span>
              <strong id="browse-lock-mc"></strong>
            </div>
            <div class="browse-locked-row">
              <span class="muted">${deps.t("browse.loader")}</span>
              <strong id="browse-lock-loader"></strong>
            </div>
            <button type="button" class="btn btn-ghost sm" id="browse-unlock">${deps.t("browse.unlock")}</button>
          </div>
          <div class="field" id="browse-free-filters" hidden>
            <label class="tiny muted">${deps.t("instance.browseMods.filterLoader")}
              <input class="search" id="browse-loader-free" />
            </label>
            <label class="tiny muted">${deps.t("instance.browseMods.filterMc")}
              <input class="search" id="browse-mc-free" />
            </label>
          </div>
          <h3 class="browse-side-title">${deps.t("browse.categories")}</h3>
          <div class="chips tiny" id="browse-cats" role="listbox"></div>
        </aside>
      </div>
    </section>
  `;

  if (instance) {
    root.querySelector("#browse-inst-name")!.textContent = instance.name;
    root.querySelector("#browse-inst-meta")!.textContent = deps.t("browse.installMeta", {
      mc: instance.minecraft,
      loader: deps.prettyLoader(instance.loader),
    });
  }

  const queryInput = root.querySelector<HTMLInputElement>("#browse-query")!;
  const sortSel = root.querySelector<HTMLSelectElement>("#browse-sort")!;
  const sizeSel = root.querySelector<HTMLSelectElement>("#browse-pagesize")!;
  const results = root.querySelector<HTMLElement>("#browse-results")!;
  const pager = root.querySelector<HTMLElement>("#browse-pager")!;
  const pageLabel = root.querySelector<HTMLElement>("#browse-page-label")!;
  const lockBox = root.querySelector<HTMLElement>("#browse-locked")!;
  const freeFilters = root.querySelector<HTMLElement>("#browse-free-filters")!;
  const lockMc = root.querySelector<HTMLElement>("#browse-lock-mc")!;
  const lockLoader = root.querySelector<HTMLElement>("#browse-lock-loader")!;
  const catsEl = root.querySelector<HTMLElement>("#browse-cats")!;

  const typeLabelKey =
    projectType === "mod"
      ? "browse.type.mods"
      : projectType === "modpack"
        ? "browse.type.modpacks"
        : projectType === "resourcepack"
          ? "browse.type.resourcepacks"
          : projectType === "datapack"
            ? "browse.type.datapacks"
            : "browse.type.shaders";
  queryInput.placeholder = deps.t("browse.searchPlaceholder", {
    type: deps.t(typeLabelKey),
  });

  for (const s of SORTS) {
    const opt = document.createElement("option");
    opt.value = s.id;
    opt.textContent = deps.t(s.key);
    sortSel.append(opt);
  }
  for (const n of PAGE_SIZES) {
    const opt = document.createElement("option");
    opt.value = String(n);
    opt.textContent = String(n);
    if (n === pageSize) opt.selected = true;
    sizeSel.append(opt);
  }

  const syncLockUi = () => {
    lockBox.hidden = !filtersLocked || !instance;
    freeFilters.hidden = filtersLocked && Boolean(instance);
    if (instance && filtersLocked) {
      lockMc.textContent = instance.minecraft;
      lockLoader.textContent = deps.prettyLoader(instance.loader);
    }
  };
  syncLockUi();

  const paintTabs = () => {
    const tabs = root.querySelector<HTMLElement>("#browse-type-tabs")!;
    tabs.replaceChildren();
    const visibleTabs = instance
      ? TYPE_TABS.filter((tab) => tab.id !== "modpack")
      : TYPE_TABS;
    for (const tab of visibleTabs) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `tab${projectType === tab.id ? " is-active" : ""}`;
      btn.setAttribute("role", "tab");
      btn.setAttribute("aria-selected", projectType === tab.id ? "true" : "false");
      btn.textContent = deps.t(tab.key);
      btn.addEventListener("click", () => {
        if (projectType === tab.id) return;
        projectType = tab.id;
        opts.onProjectTypeChange(tab.id);
        offset = 0;
        paintTabs();
        paintCats();
        queryInput.placeholder = deps.t("browse.searchPlaceholder", {
          type: deps.t(tab.key),
        });
        void runSearch();
      });
      tabs.append(btn);
    }
  };

  const paintCats = () => {
    catsEl.replaceChildren();
    const list = projectType === "mod" ? MOD_CATEGORIES : [""];
    for (const id of list) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `chip${category === id ? " is-selected" : ""}`;
      btn.setAttribute("role", "option");
      btn.setAttribute("aria-selected", category === id ? "true" : "false");
      btn.textContent = id
        ? deps.t(`instance.browseMods.cat.${id}`)
        : deps.t("instance.browseMods.cat.all");
      btn.addEventListener("click", () => {
        category = id;
        offset = 0;
        paintCats();
        void runSearch();
      });
      catsEl.append(btn);
    }
  };

  const effectiveLoader = (): string => {
    if (filtersLocked && instance) return instance.loader;
    return root.querySelector<HTMLInputElement>("#browse-loader-free")?.value.trim() || loaderLocked || "fabric";
  };
  const effectiveMc = (): string => {
    if (filtersLocked && instance) return instance.minecraft;
    return root.querySelector<HTMLInputElement>("#browse-mc-free")?.value.trim() || mcLocked || "";
  };

  const paintHits = (hits: BrowseHit[]) => {
    results.replaceChildren();
    const filtered = hideInstalled
      ? hits.filter((h) => !installedIds.has(h.projectId))
      : hits;
    if (!filtered.length) {
      results.innerHTML = `<div class="empty-state"><h3>${deps.t("browse.noResults")}</h3><p class="muted">${deps.t("browse.noResultsHint")}</p></div>`;
      return;
    }
    for (const hit of filtered) {
      const card = document.createElement("article");
      card.className = "browse-card";
      card.innerHTML = `
        <button type="button" class="browse-card-open">
          <span class="browse-card-icon"></span>
          <span class="browse-card-body">
            <strong class="browse-card-title"></strong>
            <span class="muted browse-card-desc"></span>
            <span class="browse-card-meta muted"></span>
          </span>
        </button>
        <div class="browse-card-actions">
          <button type="button" class="btn btn-primary sm browse-install">${deps.t("instance.browseMods.install")}</button>
        </div>
      `;
      const icon = card.querySelector<HTMLElement>(".browse-card-icon")!;
      if (hit.iconUrl && deps.isSafeModrinthMediaUrl(hit.iconUrl)) {
        icon.style.backgroundImage = `url(${JSON.stringify(hit.iconUrl)})`;
        icon.classList.add("has-img");
      } else {
        icon.textContent = hit.title.slice(0, 1).toUpperCase();
      }
      card.querySelector(".browse-card-title")!.textContent = hit.title;
      card.querySelector(".browse-card-desc")!.textContent = hit.description;
      card.querySelector(".browse-card-meta")!.textContent = [
        formatDownloads(hit.downloads),
        ...(hit.categories ?? []).slice(0, 3),
      ].join(" · ");
      const installed = installedIds.has(hit.projectId);
      const installBtn = card.querySelector<HTMLButtonElement>(".browse-install")!;
      const isModpack = projectType === "modpack" || hit.projectType === "modpack";
      if (installed) {
        installBtn.textContent = deps.t("instance.browseMods.installed");
        installBtn.classList.add("is-selected");
        installBtn.disabled = true;
      } else if (!instance && !isModpack) {
        installBtn.textContent = deps.t("browse.pickInstance");
        installBtn.disabled = true;
        installBtn.title = deps.t("browse.pickInstanceHint");
      } else if (!instance && isModpack) {
        installBtn.textContent = deps.t("browse.installModpack");
        installBtn.disabled = false;
      }
      card.querySelector(".browse-card-open")?.addEventListener("click", () => {
        deps.onOpenProject(hit, projectType);
      });
      installBtn.addEventListener("click", () => {
        if (busy || installed) return;
        if (isModpack && !instance) {
          void installModpack(hit, installBtn);
          return;
        }
        if (!instance) return;
        void installProject(hit, installBtn);
      });
      results.append(card);
    }
  };

  const updatePager = () => {
    const page = Math.floor(offset / pageSize) + 1;
    const pages = Math.max(1, Math.ceil(total / pageSize));
    pager.hidden = total <= pageSize;
    pageLabel.textContent = deps.t("browse.pageOf", { page, pages, total });
    root.querySelector<HTMLButtonElement>("#browse-prev")!.disabled = offset <= 0;
    root.querySelector<HTMLButtonElement>("#browse-next")!.disabled = offset + pageSize >= total;
  };

  async function runSearch() {
    const gen = ++searchGen;
    results.textContent = deps.t("browse.searching");
    if (!deps.inTauri) {
      results.innerHTML = `<p class="muted">${deps.t("instance.browseMods.stub")}</p>`;
      pager.hidden = true;
      return;
    }
    try {
      if (deps.getInstalledProjectIds && instance) {
        installedIds = await deps.getInstalledProjectIds();
      }
      const loader = effectiveLoader();
      const mc = effectiveMc();
      const res = await invoke<{
        hits: BrowseHit[];
        offset: number;
        limit: number;
        totalHits: number;
        stub?: boolean;
        message?: string;
      }>("modrinth_search", {
        query: query.trim(),
        limit: pageSize,
        offset,
        loader: loader && loader !== "vanilla" && projectType === "mod" ? loader : null,
        gameVersion: mc || null,
        category: category || null,
        projectType,
        index: sort,
      });
      if (gen !== searchGen) return;
      total = res.totalHits ?? 0;
      if (res.stub) {
        results.innerHTML = `<p class="muted">${escapeHtml(res.message || deps.t("instance.browseMods.stub"))}</p>`;
        pager.hidden = true;
        return;
      }
      paintHits(res.hits ?? []);
      updatePager();
    } catch (e) {
      if (gen !== searchGen) return;
      results.innerHTML = `<p class="status is-err">${escapeHtml(deps.friendlyError(e))}</p>`;
      pager.hidden = true;
    }
  }

  async function installModpack(hit: BrowseHit, btn: HTMLButtonElement) {
    busy = true;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = deps.t("instance.browseMods.installing");
    try {
      const result = await invoke<{
        instance: { id: string };
        message: string;
        partial?: boolean;
      }>("import_mrpack_modrinth", {
        projectId: hit.projectId,
        versionId: null,
      });
      installedIds.add(hit.projectId);
      btn.textContent = deps.t("instance.browseMods.installed");
      btn.classList.add("is-selected");
      deps.setStatus(result.message || deps.t("browse.modpackImported", { name: hit.title }));
      await deps.onModpackImported?.(result.instance.id);
    } catch (e) {
      btn.disabled = false;
      btn.textContent = prev || deps.t("browse.installModpack");
      deps.setStatus(deps.friendlyError(e));
    } finally {
      busy = false;
    }
  }

  async function installProject(hit: BrowseHit, btn: HTMLButtonElement) {
    if (!instance) return;
    busy = true;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = deps.t("instance.browseMods.installing");
    try {
      await invoke("modrinth_install", {
        instanceId: instance.id,
        projectId: hit.projectId,
        loader: effectiveLoader(),
        gameVersion: effectiveMc(),
        includeOptionalDependencies: optionalDeps,
      });
      installedIds.add(hit.projectId);
      btn.textContent = deps.t("instance.browseMods.installed");
      btn.classList.add("is-selected");
      deps.setStatus(deps.t("instance.browseMods.installedMsg", { name: hit.title }));
    } catch (e) {
      btn.disabled = false;
      btn.textContent = prev || deps.t("instance.browseMods.install");
      deps.setStatus(deps.friendlyError(e));
    } finally {
      busy = false;
    }
  }

  root.querySelector("#browse-back")?.addEventListener("click", () => {
    deps.onBackToContent?.();
  });
  root.querySelector("#browse-unlock")?.addEventListener("click", () => {
    filtersLocked = !filtersLocked;
    root.querySelector("#browse-unlock")!.textContent = filtersLocked
      ? deps.t("browse.unlock")
      : deps.t("browse.lock");
    syncLockUi();
    if (instance && !filtersLocked) {
      const lf = root.querySelector<HTMLInputElement>("#browse-loader-free")!;
      const mf = root.querySelector<HTMLInputElement>("#browse-mc-free")!;
      lf.value = instance.loader;
      mf.value = instance.minecraft;
    }
    offset = 0;
    void runSearch();
  });
  root.querySelector<HTMLInputElement>("#browse-hide-installed")!.addEventListener("change", (e) => {
    hideInstalled = (e.target as HTMLInputElement).checked;
    void runSearch();
  });
  root.querySelector<HTMLInputElement>("#browse-optional-deps")!.addEventListener("change", (e) => {
    optionalDeps = (e.target as HTMLInputElement).checked;
  });
  sortSel.addEventListener("change", () => {
    sort = sortSel.value;
    offset = 0;
    void runSearch();
  });
  sizeSel.addEventListener("change", () => {
    pageSize = Number(sizeSel.value) as (typeof PAGE_SIZES)[number];
    offset = 0;
    void runSearch();
  });
  queryInput.addEventListener("input", () => {
    query = queryInput.value;
    window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => {
      offset = 0;
      void runSearch();
    }, 400);
  });
  queryInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      window.clearTimeout(searchTimer);
      offset = 0;
      void runSearch();
    }
  });
  root.querySelector("#browse-prev")?.addEventListener("click", () => {
    offset = Math.max(0, offset - pageSize);
    void runSearch();
  });
  root.querySelector("#browse-next")?.addEventListener("click", () => {
    offset += pageSize;
    void runSearch();
  });

  paintTabs();
  paintCats();
  void runSearch();
}

export type ProjectDetailDeps = BrowseDeps & {
  instance: BrowseInstanceCtx | null;
  projectType: BrowseProjectType;
  onBack: () => void;
};

type ProjectDetail = {
  id: string;
  title: string;
  slug: string;
  description: string;
  body?: string;
  downloads: number;
  projectType: string;
  iconUrl?: string | null;
  categories?: string[];
  loaders?: string[];
  gameVersions?: string[];
  gallery?: Array<{ url: string; title?: string | null }>;
  projectUrl?: string | null;
};

type ProjectVersionRow = {
  id: string;
  name: string;
  versionNumber: string;
  versionType: string;
  loaders: string[];
  gameVersions: string[];
  downloads?: number;
  datePublished?: string;
  changelog?: string | null;
};

export function mountProjectPage(root: HTMLElement, hit: BrowseHit, deps: ProjectDetailDeps): void {
  let activeTab: "description" | "gallery" | "versions" | "changelog" = "description";
  let optionalDeps = false;
  let busy = false;
  let projectGen = 0;
  let versionsGen = 0;
  let projectCache: ProjectDetail | null = null;
  let versionsCache: ProjectVersionRow[] | null = null;
  let versionsLoading = false;
  const versionsWaiters: Array<() => void> = [];

  root.innerHTML = `
    <section class="page project-page">
      <div class="mr-detail-shell">
        <div class="mr-detail-main">
          <div class="mr-detail-head">
            <button type="button" class="btn btn-ghost sm" id="proj-back">${deps.t("instance.browseMods.back")}</button>
            <span class="project-icon-lg" id="proj-icon"></span>
            <div>
              <h1 class="mr-detail-title" id="proj-title"></h1>
              <p class="muted" id="proj-desc"></p>
            </div>
            <button type="button" class="btn btn-primary" id="proj-install">${
              deps.projectType === "modpack" && !deps.instance
                ? deps.t("browse.installModpack")
                : deps.t("instance.browseMods.install")
            }</button>
          </div>
          <div class="mr-detail-tabs" role="tablist">
            <button type="button" class="tab is-active" data-ptab="description">${deps.t("instance.browseMods.tab.description")}</button>
            <button type="button" class="tab" data-ptab="gallery">${deps.t("instance.browseMods.tab.gallery")}</button>
            <button type="button" class="tab" data-ptab="changelog">${deps.t("instance.browseMods.tab.changelog")}</button>
            <button type="button" class="tab" data-ptab="versions">${deps.t("instance.browseMods.tab.versions")}</button>
          </div>
          <div class="mr-detail-pane" id="proj-pane"></div>
        </div>
        <aside class="mr-detail-sidebar">
          <h4>${deps.t("instance.browseMods.sidebar.info")}</h4>
          <dl class="mr-side-dl">
            <div><dt>${deps.t("instance.browseMods.sidebar.type")}</dt><dd id="proj-side-type"></dd></div>
            <div><dt>${deps.t("instance.browseMods.sidebar.downloads")}</dt><dd id="proj-side-dl"></dd></div>
            <div><dt>${deps.t("instance.browseMods.sidebar.loaders")}</dt><dd id="proj-side-loaders"></dd></div>
            <div><dt>${deps.t("instance.browseMods.sidebar.mc")}</dt><dd id="proj-side-mc"></dd></div>
            <div><dt>${deps.t("instance.browseMods.sidebar.categories")}</dt><dd id="proj-side-cats"></dd></div>
          </dl>
          <label class="check-row tiny">
            <input type="checkbox" id="proj-optional" />
            <span>${deps.t("instance.browseMods.optionalDeps")}</span>
          </label>
          <a class="btn btn-ghost sm" id="proj-modrinth" href="#" target="_blank" rel="noopener noreferrer">${deps.t("instance.browseMods.sidebar.openModrinth")}</a>
        </aside>
      </div>
    </section>
  `;

  root.querySelector("#proj-title")!.textContent = hit.title;
  root.querySelector("#proj-desc")!.textContent = hit.description;
  root.querySelector("#proj-side-type")!.textContent = hit.projectType;
  root.querySelector("#proj-side-dl")!.textContent = formatDownloads(hit.downloads);
  const icon = root.querySelector<HTMLElement>("#proj-icon")!;
  if (hit.iconUrl && deps.isSafeModrinthMediaUrl(hit.iconUrl)) {
    icon.style.backgroundImage = `url(${JSON.stringify(hit.iconUrl)})`;
    icon.classList.add("has-img");
  } else {
    icon.textContent = hit.title.slice(0, 1).toUpperCase();
  }

  if (!deps.instance) {
    const btn = root.querySelector<HTMLButtonElement>("#proj-install")!;
    btn.disabled = true;
    btn.title = deps.t("browse.pickInstanceHint");
  }

  const ensureVersions = (afterLoad: () => void) => {
    if (versionsCache) {
      afterLoad();
      return;
    }
    if (!deps.inTauri) {
      versionsCache = [];
      afterLoad();
      return;
    }
    versionsWaiters.push(afterLoad);
    if (versionsLoading) return;
    versionsLoading = true;
    const gen = ++versionsGen;
    const flushWaiters = () => {
      versionsLoading = false;
      const waiters = versionsWaiters.splice(0, versionsWaiters.length);
      for (const w of waiters) w();
    };
    void invoke<ProjectVersionRow[]>("modrinth_project_versions", {
      projectId: hit.projectId,
      loader: deps.instance?.loader ?? null,
      gameVersion: deps.instance?.minecraft ?? null,
    })
      .then((rows) => {
        if (gen === versionsGen) {
          versionsCache = rows;
        }
        // Always flush — stale gen must not leave Changelog stuck on "searching".
        flushWaiters();
      })
      .catch(() => {
        if (gen === versionsGen) {
          versionsCache = [];
        }
        flushWaiters();
      });
  };

  const paintPane = () => {
    const pane = root.querySelector<HTMLElement>("#proj-pane")!;
    root.querySelectorAll<HTMLButtonElement>("[data-ptab]").forEach((b) => {
      b.classList.toggle("is-active", b.dataset.ptab === activeTab);
    });
    if (activeTab === "description") {
      const body = projectCache?.body || projectCache?.description || hit.description;
      pane.innerHTML = `<div class="mr-detail-body">${escapeHtml(body).replace(/\n/g, "<br>")}</div>`;
    } else if (activeTab === "gallery") {
      const gallery = (projectCache?.gallery ?? []).filter((g) =>
        deps.isSafeModrinthMediaUrl(g.url),
      );
      if (!gallery.length) {
        pane.innerHTML = `<p class="muted">${deps.t("instance.browseMods.galleryEmpty")}</p>`;
      } else {
        pane.innerHTML = `<div class="mr-gallery">${gallery
          .map(
            (g) =>
              `<a href="${escapeAttr(g.url)}" target="_blank" rel="noopener noreferrer"><img src="${escapeAttr(g.url)}" alt="${escapeAttr(g.title || hit.title)}" /></a>`,
          )
          .join("")}</div>`;
      }
    } else if (activeTab === "changelog") {
      if (!versionsCache) {
        pane.innerHTML = `<p class="muted">${deps.t("browse.searching")}</p>`;
        ensureVersions(() => {
          if (activeTab === "changelog") paintPane();
        });
        return;
      }
      const entries = versionsCache.filter((v) => (v.changelog || "").trim().length > 0);
      if (!entries.length) {
        pane.innerHTML = `<p class="muted">${deps.t("instance.browseMods.changelogEmpty")}</p>`;
        return;
      }
      pane.innerHTML = `<div class="mr-changelog">${entries
        .slice(0, 40)
        .map((v) => {
          const title = escapeHtml(v.versionNumber || v.name);
          const meta = escapeHtml(
            [v.versionType, v.datePublished ? v.datePublished.slice(0, 10) : ""]
              .filter(Boolean)
              .join(" · "),
          );
          const body = escapeHtml(v.changelog || "").replace(/\n/g, "<br>");
          return `<article class="mr-changelog-entry"><h3>${title}</h3><p class="muted tiny">${meta}</p><div class="mr-detail-body">${body}</div></article>`;
        })
        .join("")}</div>`;
    } else {
      if (!versionsCache) {
        pane.innerHTML = `<p class="muted">${deps.t("browse.searching")}</p>`;
        ensureVersions(() => {
          if (activeTab === "versions") paintPane();
        });
        return;
      }
      if (!versionsCache.length) {
        pane.innerHTML = `<p class="muted">${deps.t("instance.browseMods.noVersions")}</p>`;
        return;
      }
      const list = document.createElement("div");
      list.className = "mr-detail-versions";
      for (const v of versionsCache) {
        const row = document.createElement("div");
        row.className = "mr-version-row";
        row.innerHTML = `
          <div>
            <strong></strong>
            <small class="muted"></small>
          </div>
          <button type="button" class="btn btn-primary sm" data-vid></button>
        `;
        row.querySelector("strong")!.textContent = v.versionNumber || v.name;
        row.querySelector("small")!.textContent = [
          v.versionType,
          v.loaders?.join(", "),
          v.gameVersions?.slice(0, 4).join(", "),
          v.downloads != null ? formatDownloads(v.downloads) : "",
        ]
          .filter(Boolean)
          .join(" · ");
        const btn = row.querySelector<HTMLButtonElement>("button")!;
        btn.textContent =
          deps.projectType === "modpack" && !deps.instance
            ? deps.t("browse.installModpack")
            : deps.t("instance.browseMods.install");
        btn.disabled = busy || (!deps.instance && deps.projectType !== "modpack");
        btn.addEventListener("click", () => void installVersion(v.id, btn));
        list.append(row);
      }
      pane.replaceChildren(list);
    }
  };

  async function installVersion(versionId: string | undefined, btn: HTMLButtonElement) {
    if (busy) return;
    const isModpack = deps.projectType === "modpack" || hit.projectType === "modpack";
    if (!deps.instance && !isModpack) return;
    busy = true;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = deps.t("instance.browseMods.installing");
    try {
      if (isModpack && !deps.instance) {
        const result = await invoke<{
          instance: { id: string };
          message: string;
        }>("import_mrpack_modrinth", {
          projectId: hit.projectId,
          versionId: versionId ?? null,
        });
        btn.textContent = deps.t("instance.browseMods.installed");
        btn.classList.add("is-selected");
        deps.setStatus(result.message || deps.t("browse.modpackImported", { name: hit.title }));
        await deps.onModpackImported?.(result.instance.id);
        return;
      }
      if (!deps.instance) return;
      if (versionId) {
        await invoke("modrinth_install_version", {
          instanceId: deps.instance.id,
          versionId,
          withDependencies: true,
          includeOptionalDependencies: optionalDeps,
        });
      } else {
        await invoke("modrinth_install", {
          instanceId: deps.instance.id,
          projectId: hit.projectId,
          loader: deps.instance.loader,
          gameVersion: deps.instance.minecraft,
          includeOptionalDependencies: optionalDeps,
        });
      }
      btn.textContent = deps.t("instance.browseMods.installed");
      btn.classList.add("is-selected");
      deps.setStatus(deps.t("instance.browseMods.installedMsg", { name: hit.title }));
    } catch (e) {
      btn.disabled = false;
      btn.textContent = prev || deps.t("instance.browseMods.install");
      deps.setStatus(deps.friendlyError(e));
    } finally {
      busy = false;
    }
  }

  root.querySelector("#proj-back")?.addEventListener("click", () => deps.onBack());
  root.querySelector("#proj-install")?.addEventListener("click", (e) => {
    void installVersion(undefined, e.currentTarget as HTMLButtonElement);
  });
  root.querySelector<HTMLInputElement>("#proj-optional")!.addEventListener("change", (e) => {
    optionalDeps = (e.target as HTMLInputElement).checked;
  });
  root.querySelectorAll<HTMLButtonElement>("[data-ptab]").forEach((b) => {
    b.addEventListener("click", () => {
      activeTab = b.dataset.ptab as typeof activeTab;
      paintPane();
    });
  });

  paintPane();

  if (deps.inTauri) {
    const gen = ++projectGen;
    void invoke<ProjectDetail>("modrinth_get_project", {
      projectId: hit.projectId,
    })
      .then((p) => {
        if (gen !== projectGen) return;
        projectCache = p;
        root.querySelector("#proj-title")!.textContent = p.title || hit.title;
        root.querySelector("#proj-desc")!.textContent = p.description || hit.description;
        root.querySelector("#proj-side-type")!.textContent = p.projectType || hit.projectType;
        root.querySelector("#proj-side-dl")!.textContent = formatDownloads(p.downloads ?? hit.downloads);
        root.querySelector("#proj-side-loaders")!.textContent = (p.loaders ?? []).join(", ") || "—";
        root.querySelector("#proj-side-mc")!.textContent =
          (p.gameVersions ?? []).slice(0, 8).join(", ") || "—";
        root.querySelector("#proj-side-cats")!.textContent = (p.categories ?? []).join(", ") || "—";
        const link = root.querySelector<HTMLAnchorElement>("#proj-modrinth")!;
        const href = p.projectUrl || `https://modrinth.com/${p.projectType || "mod"}/${p.slug || hit.slug}`;
        link.href = deps.isSafeModrinthMediaUrl(href) ? href : "#";
        if (activeTab === "description" || activeTab === "gallery") paintPane();
      })
      .catch(() => {
        /* keep hit data */
      });
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeAttr(s: string): string {
  return escapeHtml(s).replace(/'/g, "&#39;");
}
