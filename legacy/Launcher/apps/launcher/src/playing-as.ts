import { t } from "./i18n";

export type ProfileKind = "offline" | "microsoft" | "owyx";
export type Profile = {
  id: string;
  kind: ProfileKind;
  name: string;
  nick: string;
  needsAuth?: boolean;
  authBlob?: string | null;
  createdAt: number;
};

type Deps = {
  getProfiles: () => Profile[];
  getActiveId: () => string | null;
  setActiveId: (id: string | null) => void;
  setProfiles: (next: Profile[]) => void;
  persist: () => Promise<void>;
  openWizard: (opts?: { allowClose?: boolean }) => void;
  setStatus: (el: HTMLElement, text: string, kind?: "ok" | "err" | "muted") => void;
  /** Called when a profile is removed, e.g. to forget a stored Owyx session. */
  onRemove?: (profile: Profile) => void;
};

function accountLabel(kind: ProfileKind) {
  return t(`profile.account.${kind}`);
}

function fillAvatar(el: HTMLElement, nick: string) {
  const letter = (nick || "?").slice(0, 1).toUpperCase();
  el.replaceChildren();
  el.textContent = letter;
  const img = document.createElement("img");
  img.alt = "";
  img.width = 40;
  img.height = 40;
  img.src = `https://mc-heads.net/avatar/${encodeURIComponent(nick || "Steve")}/40`;
  img.addEventListener("load", () => el.replaceChildren(img));
  img.addEventListener("error", () => {
    el.textContent = letter;
  });
}

let closer: ((e: MouseEvent) => void) | null = null;

export function mountPlayingAs(host: HTMLElement, deps: Deps) {
  if (closer) {
    window.removeEventListener("click", closer);
    closer = null;
  }

  host.innerHTML = `
    <div class="playing-as">
      <div class="playing-as-label">${t("profile.playingAs")}</div>
      <div class="playing-as-menu">
        <button type="button" class="playing-as-trigger" id="playing-as-trigger" aria-expanded="false">
          <span class="playing-as-avatar" id="playing-as-avatar"></span>
          <span class="playing-as-meta">
            <strong id="playing-as-name"></strong>
            <small id="playing-as-kind"></small>
          </span>
          <span class="playing-as-chevron" aria-hidden="true">▾</span>
        </button>
        <div class="playing-as-panel" id="playing-as-panel" hidden></div>
      </div>
      <p id="home-status" class="status"></p>
    </div>
  `;

  const trigger = host.querySelector<HTMLButtonElement>("#playing-as-trigger")!;
  const panel = host.querySelector<HTMLElement>("#playing-as-panel")!;
  const status = host.querySelector<HTMLElement>("#home-status")!;
  const avatar = host.querySelector<HTMLElement>("#playing-as-avatar")!;

  const close = () => {
    panel.hidden = true;
    trigger.classList.remove("is-open");
    trigger.setAttribute("aria-expanded", "false");
  };

  const openAdd = (allowClose: boolean) => {
    close();
    // Defer so the same click does not immediately dismiss the modal.
    window.setTimeout(() => deps.openWizard({ allowClose }), 0);
  };

  const active = () =>
    deps.getProfiles().find((p) => p.id === deps.getActiveId()) ??
    deps.getProfiles()[0] ??
    null;

  const paintTrigger = () => {
    const p = active();
    fillAvatar(avatar, p?.nick || p?.name || "Steve");
    host.querySelector("#playing-as-name")!.textContent = p?.name ?? t("profile.none");
    host.querySelector("#playing-as-kind")!.textContent = p ? accountLabel(p.kind) : "—";
    if (p?.needsAuth) deps.setStatus(status, t("profile.needsAuth"), "muted");
    else setStatusEmpty();
  };

  const setStatusEmpty = () => deps.setStatus(status, "", "muted");

  const paintPanel = () => {
    panel.replaceChildren();
    for (const p of deps.getProfiles()) {
      const wrap = document.createElement("div");
      wrap.className = "playing-as-item";

      const pick = document.createElement("button");
      pick.type = "button";
      pick.className = `playing-as-row${p.id === deps.getActiveId() ? " is-active" : ""}`;
      pick.innerHTML = `
        <span class="playing-as-radio" aria-hidden="true"></span>
        <span class="playing-as-avatar" data-avatar></span>
        <span class="playing-as-meta"><strong></strong><small></small></span>
      `;
      fillAvatar(pick.querySelector("[data-avatar]")!, p.nick || p.name);
      pick.querySelector("strong")!.textContent = p.name;
      pick.querySelector("small")!.textContent = accountLabel(p.kind);
      pick.addEventListener("click", (e) => {
        e.stopPropagation();
        deps.setActiveId(p.id);
        void deps.persist().then(() => {
          paintTrigger();
          paintPanel();
          close();
        });
      });

      const del = document.createElement("button");
      del.type = "button";
      del.className = "playing-as-del";
      del.title = t("profile.delete");
      del.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 3h6l1 2h4v2H4V5h4l1-2Zm1 6h2v9h-2V9Zm4 0h2v9h-2V9ZM7 9h2v9H7V9Zm-1 12h12l1-12H5l1 12Z"/></svg>`;
      del.addEventListener("click", (e) => {
        e.stopPropagation();
        deps.onRemove?.(p);
        const next = deps.getProfiles().filter((x) => x.id !== p.id);
        deps.setProfiles(next);
        if (deps.getActiveId() === p.id) deps.setActiveId(next[0]?.id ?? null);
        void deps.persist().then(() => {
          paintTrigger();
          paintPanel();
          if (!next.length) openAdd(false);
          else close();
        });
      });

      wrap.append(pick, del);
      panel.append(wrap);
    }

    const add = document.createElement("button");
    add.type = "button";
    add.className = "playing-as-add";
    add.innerHTML = `<span aria-hidden="true">+</span> ${t("profile.add")}`;
    add.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      openAdd(deps.getProfiles().length > 0);
    });
    panel.append(add);
  };

  trigger.addEventListener("click", (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (panel.hidden) {
      paintPanel();
      panel.hidden = false;
      trigger.classList.add("is-open");
      trigger.setAttribute("aria-expanded", "true");
    } else close();
  });
  panel.addEventListener("click", (e) => e.stopPropagation());

  closer = (e: MouseEvent) => {
    const target = e.target as Node | null;
    if (target && host.contains(target)) return;
    close();
  };
  window.addEventListener("click", closer);
  paintTrigger();
}
