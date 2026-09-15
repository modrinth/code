/**
 * Skins screen — Modrinth Skins.vue spirit + Owyx /me skin-to-disk.
 * List / preview / apply / import / remove against Rust skin library.
 */

export type SkinEntry = {
  id: string;
  name: string;
  model: string;
  source: string;
  path: string;
  textureDataUrl?: string | null;
  isEquipped: boolean;
  createdAt?: number;
};

export type SkinsDeps = {
  t: (key: string, vars?: Record<string, string | number>) => string;
  inTauri: boolean;
  activeNick: string | null;
  owyxSkinUrl: string | null;
  listSkins: () => Promise<SkinEntry[]>;
  applySkin: (id: string) => Promise<void>;
  removeSkin: (id: string) => Promise<void>;
  onApplyOwyxSkin: () => Promise<void>;
  onImportFile: () => Promise<void>;
  setStatus: (msg: string, kind?: "ok" | "err" | "muted") => void;
};

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** CSS face crop from a 64×64 Minecraft skin texture. */
function faceStyle(textureUrl: string | null | undefined): string {
  if (!textureUrl) return "";
  const safe = JSON.stringify(textureUrl);
  return `background-image:url(${safe});background-size:800% 800%;background-position:12.5% 12.5%;`;
}

export function mountSkinsPage(root: HTMLElement, deps: SkinsDeps): void {
  root.innerHTML = `
    <section class="page skins-page">
      <header class="page-head">
        <h1>${deps.t("skins.title")}</h1>
        <p class="muted">${deps.t("skins.subtitle")}</p>
      </header>
      <div class="skins-layout">
        <aside class="skins-preview-panel">
          <div class="skins-preview" id="skins-preview" aria-hidden="true">
            <div class="skins-preview-figure" id="skins-preview-figure"></div>
            <div class="skins-preview-face" id="skins-preview-face"></div>
          </div>
          <p class="muted tiny" id="skins-preview-label"></p>
          <div class="skins-preview-actions">
            <button type="button" class="btn btn-primary" id="skins-apply-selected" disabled>${deps.t("skins.apply")}</button>
            <button type="button" class="btn btn-ghost sm" id="skins-remove-selected" hidden>${deps.t("skins.remove")}</button>
          </div>
        </aside>
        <div class="skins-list-panel">
          <div class="skins-section">
            <div class="skins-section-head">
              <h2>${deps.t("skins.saved")}</h2>
              <button type="button" class="btn btn-ghost sm" id="skins-import">${deps.t("skins.import")}</button>
            </div>
            <div class="skins-grid" id="skins-grid"></div>
          </div>
          <div class="skins-section">
            <h2>${deps.t("skins.owyx")}</h2>
            <div class="skins-owyx-card">
              <div class="skins-owyx-row">
                <div class="skins-owyx-thumb" id="skins-owyx-thumb"></div>
                <div>
                  <p class="muted">${deps.t("skins.owyxHint")}</p>
                  <button type="button" class="btn btn-primary" id="skins-apply-owyx">${deps.t("skins.applyOwyx")}</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  `;

  const previewLabel = root.querySelector<HTMLElement>("#skins-preview-label")!;
  const figure = root.querySelector<HTMLElement>("#skins-preview-figure")!;
  const face = root.querySelector<HTMLElement>("#skins-preview-face")!;
  const grid = root.querySelector<HTMLElement>("#skins-grid")!;
  const applyBtn = root.querySelector<HTMLButtonElement>("#skins-apply-selected")!;
  const removeBtn = root.querySelector<HTMLButtonElement>("#skins-remove-selected")!;
  const owyxThumb = root.querySelector<HTMLElement>("#skins-owyx-thumb")!;

  let skins: SkinEntry[] = [];
  let selectedId: string | null = null;

  const nick = deps.activeNick || deps.t("skins.noProfile");
  previewLabel.textContent = deps.t("skins.playingAs", { nick });
  figure.textContent = nick.slice(0, 1).toUpperCase();

  if (deps.owyxSkinUrl) {
    owyxThumb.style.cssText = faceStyle(deps.owyxSkinUrl);
    owyxThumb.classList.add("has-img");
  }

  function paintPreview(skin: SkinEntry | undefined) {
    if (!skin) {
      face.removeAttribute("style");
      face.classList.remove("has-img");
      figure.style.display = "";
      return;
    }
    const tex = skin.textureDataUrl || null;
    if (tex) {
      face.style.cssText = faceStyle(tex);
      face.classList.add("has-img");
      figure.style.display = "none";
    } else {
      face.removeAttribute("style");
      face.classList.remove("has-img");
      figure.style.display = "";
      figure.textContent = skin.name.slice(0, 1).toUpperCase();
    }
    previewLabel.textContent = `${skin.name} · ${skin.model}`;
    applyBtn.disabled = skin.isEquipped;
    applyBtn.textContent = skin.isEquipped ? deps.t("skins.equipped") : deps.t("skins.apply");
    const canRemove = skin.source === "custom";
    removeBtn.hidden = !canRemove;
  }

  function renderGrid() {
    const tiles = skins
      .map((s) => {
        const selected = s.id === selectedId ? " is-selected" : "";
        const equipped = s.isEquipped ? `<span class="skins-tile-badge">${deps.t("skins.equipped")}</span>` : "";
        const thumb = s.textureDataUrl
          ? `<span class="skins-tile-icon has-img" style="${faceStyle(s.textureDataUrl)}"></span>`
          : `<span class="skins-tile-icon">${escapeHtml(s.name.slice(0, 1).toUpperCase())}</span>`;
        return `<button type="button" class="skins-tile${selected}" data-skin-id="${escapeHtml(s.id)}">
          ${thumb}
          <span>${escapeHtml(s.name)}</span>
          ${equipped}
        </button>`;
      })
      .join("");
    grid.innerHTML = `
      ${tiles}
      <button type="button" class="skins-tile skins-tile-add" id="skins-add-tile">
        <span class="skins-tile-icon">+</span>
        <span>${deps.t("skins.add")}</span>
      </button>
    `;
    grid.querySelectorAll<HTMLButtonElement>("[data-skin-id]").forEach((btn) => {
      btn.addEventListener("click", () => {
        selectedId = btn.dataset.skinId || null;
        paintPreview(skins.find((s) => s.id === selectedId));
        renderGrid();
      });
    });
    grid.querySelector("#skins-add-tile")?.addEventListener("click", () => {
      void deps.onImportFile().then(() => void reload());
    });
  }

  async function reload() {
    try {
      skins = deps.inTauri ? await deps.listSkins() : defaultBrowserSkins(deps);
      selectedId = skins.find((s) => s.isEquipped)?.id ?? skins[0]?.id ?? null;
      paintPreview(skins.find((s) => s.id === selectedId));
      renderGrid();
    } catch (e) {
      deps.setStatus(String(e), "err");
      skins = defaultBrowserSkins(deps);
      selectedId = skins[0]?.id ?? null;
      paintPreview(skins[0]);
      renderGrid();
    }
  }

  root.querySelector("#skins-import")?.addEventListener("click", () => {
    void deps.onImportFile().then(() => void reload());
  });
  applyBtn.addEventListener("click", () => {
    if (!selectedId) return;
    void (async () => {
      try {
        await deps.applySkin(selectedId!);
        deps.setStatus(deps.t("skins.applied"), "ok");
        await reload();
      } catch (e) {
        deps.setStatus(String(e), "err");
      }
    })();
  });
  removeBtn.addEventListener("click", () => {
    if (!selectedId) return;
    void (async () => {
      try {
        await deps.removeSkin(selectedId!);
        deps.setStatus(deps.t("skins.removed"), "ok");
        await reload();
      } catch (e) {
        deps.setStatus(String(e), "err");
      }
    })();
  });
  root.querySelector("#skins-apply-owyx")?.addEventListener("click", () => {
    void (async () => {
      try {
        await deps.onApplyOwyxSkin();
        deps.setStatus(deps.t("skins.applied"), "ok");
        await reload();
      } catch (e) {
        deps.setStatus(String(e), "err");
      }
    })();
  });

  void reload();
}

function defaultBrowserSkins(deps: SkinsDeps): SkinEntry[] {
  return [
    {
      id: "default:steve",
      name: deps.t("skins.defaultSteve"),
      model: "classic",
      source: "default",
      path: "",
      textureDataUrl: deps.owyxSkinUrl,
      isEquipped: true,
    },
  ];
}
