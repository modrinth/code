/** Modrinth-like searchable version combobox (compact dropdown below the field). */

export type VersionComboboxOptions = {
  placeholder?: string;
  searchPlaceholder?: string;
  toggleLabel?: string;
  emptyLabel?: string;
  /** Footer toggle for snapshots / all versions (Minecraft picker). */
  showAllToggle?: boolean;
  showAllLabel?: string;
  hideAllLabel?: string;
  getShowAll?: () => boolean;
  onShowAllChange?: (showAll: boolean) => void;
  maxHeight?: number;
  onChange?: (value: string) => void;
};

export type VersionCombobox = {
  root: HTMLElement;
  getValue: () => string;
  setValue: (value: string, silent?: boolean) => void;
  setOptions: (options: string[], selected?: string) => void;
  setLabels: (labels: {
    showAll?: string;
    hideAll?: string;
    placeholder?: string;
    toggle?: string;
    empty?: string;
  }) => void;
  close: () => void;
  focus: () => void;
  destroy: () => void;
};

let comboboxSequence = 0;

export function createVersionCombobox(opts: VersionComboboxOptions = {}): VersionCombobox {
  const maxHeight = opts.maxHeight ?? 220;
  let allOptions: string[] = [];
  let value = "";
  let open = false;
  let filter = "";
  let showAllLabel = opts.showAllLabel || "Show all versions";
  let hideAllLabel = opts.hideAllLabel || "Hide snapshots";
  let toggleLabel = opts.toggleLabel || "Toggle version list";
  let emptyLabel = opts.emptyLabel || "No versions";
  let showAll = opts.getShowAll?.() ?? false;
  let onChange = opts.onChange;
  let activeIndex = -1;
  /** Skip the next focus→open when we closed via Escape / outside click. */
  let suppressOpenOnFocus = false;

  const root = document.createElement("div");
  root.className = "vcombo";

  const trigger = document.createElement("div");
  trigger.className = "vcombo-trigger";

  const input = document.createElement("input");
  input.type = "text";
  input.className = "vcombo-input";
  input.autocomplete = "off";
  input.spellcheck = false;
  input.setAttribute("role", "combobox");
  input.setAttribute("aria-expanded", "false");
  input.setAttribute("aria-autocomplete", "list");
  input.setAttribute("aria-haspopup", "listbox");
  input.placeholder = opts.searchPlaceholder || opts.placeholder || "Search…";

  const chevron = document.createElement("button");
  chevron.type = "button";
  chevron.className = "vcombo-chevron";
  chevron.setAttribute("aria-label", toggleLabel);
  chevron.tabIndex = -1;
  chevron.textContent = "▾";

  trigger.append(input, chevron);

  const menu = document.createElement("div");
  menu.className = "vcombo-menu";
  menu.hidden = true;
  menu.setAttribute("role", "listbox");
  const listId = `vcombo-list-${++comboboxSequence}`;
  menu.id = listId;
  input.setAttribute("aria-controls", listId);

  const list = document.createElement("div");
  list.className = "vcombo-list";
  list.style.maxHeight = `${maxHeight}px`;

  menu.append(list);

  let footer: HTMLButtonElement | null = null;
  if (opts.showAllToggle) {
    footer = document.createElement("button");
    footer.type = "button";
    footer.className = "vcombo-footer";
    menu.append(footer);
  }

  root.append(trigger, menu);

  const filtered = () => {
    const q = filter.trim().toLowerCase();
    if (!q) return allOptions;
    return allOptions.filter((o) => o.toLowerCase().includes(q));
  };

  const paintFooter = () => {
    if (!footer) return;
    const showing = opts.getShowAll?.() ?? showAll;
    showAll = showing;
    footer.textContent = showing ? hideAllLabel : showAllLabel;
  };

  const paintList = () => {
    list.replaceChildren();
    const items = filtered();
    if (!items.length) {
      activeIndex = -1;
      input.removeAttribute("aria-activedescendant");
      const empty = document.createElement("div");
      empty.className = "vcombo-empty";
      empty.textContent = emptyLabel;
      list.append(empty);
      return;
    }
    if (activeIndex < 0 || activeIndex >= items.length) {
      const selectedIndex = items.indexOf(value);
      activeIndex = selectedIndex >= 0 ? selectedIndex : 0;
    }
    items.forEach((item, index) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.id = `${listId}-option-${index}`;
      btn.tabIndex = -1;
      btn.className = `vcombo-option${item === value ? " is-selected" : ""}${index === activeIndex ? " is-active" : ""}`;
      btn.setAttribute("role", "option");
      btn.setAttribute("aria-selected", item === value ? "true" : "false");
      btn.textContent = item;
      btn.addEventListener("mousemove", () => {
        if (activeIndex === index) return;
        activeIndex = index;
        paintList();
      });
      btn.addEventListener("mousedown", (e) => {
        e.preventDefault();
        setValue(item);
        close({ blur: true });
      });
      list.append(btn);
    });
    input.setAttribute("aria-activedescendant", `${listId}-option-${activeIndex}`);
  };

  const openMenu = () => {
    if (open) return;
    open = true;
    root.classList.add("is-open");
    menu.hidden = false;
    input.setAttribute("aria-expanded", "true");
    filter = "";
    input.value = value;
    activeIndex = Math.max(0, allOptions.indexOf(value));
    paintFooter();
    paintList();
  };

  const close = (optsClose: { blur?: boolean } = {}) => {
    if (!open) {
      if (optsClose.blur) input.blur();
      return;
    }
    open = false;
    root.classList.remove("is-open");
    menu.hidden = true;
    input.setAttribute("aria-expanded", "false");
    input.removeAttribute("aria-activedescendant");
    filter = "";
    input.value = value;
    if (optsClose.blur) {
      suppressOpenOnFocus = true;
      input.blur();
    }
  };

  const toggle = () => {
    if (open) close();
    else {
      openMenu();
      input.focus();
      input.select();
    }
  };

  const setValue = (next: string, silent = false) => {
    value = next;
    if (!open) input.value = value;
    if (!silent) onChange?.(value);
    if (open) paintList();
  };

  const setOptions = (options: string[], selected?: string) => {
    allOptions = options.slice();
    const pick =
      selected && options.includes(selected)
        ? selected
        : options.includes(value)
          ? value
          : (options[0] ?? "");
    setValue(pick, true);
    activeIndex = Math.max(0, allOptions.indexOf(value));
    input.value = value;
    if (open) paintList();
  };

  input.addEventListener("focus", () => {
    if (suppressOpenOnFocus) {
      suppressOpenOnFocus = false;
      return;
    }
    openMenu();
    input.select();
  });
  input.addEventListener("click", () => {
    if (!open) openMenu();
  });
  input.addEventListener("input", () => {
    if (!open) openMenu();
    filter = input.value;
    activeIndex = 0;
    paintList();
  });
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      close({ blur: true });
      return;
    }
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (!open) openMenu();
      const items = filtered();
      if (items.length) {
        const direction = e.key === "ArrowDown" ? 1 : -1;
        activeIndex = (activeIndex + direction + items.length) % items.length;
        paintList();
        list
          .querySelector<HTMLElement>(`#${listId}-option-${activeIndex}`)
          ?.scrollIntoView({ block: "nearest" });
      }
      return;
    }
    if (e.key === "Home" || e.key === "End") {
      e.preventDefault();
      if (!open) openMenu();
      const items = filtered();
      if (items.length) {
        activeIndex = e.key === "Home" ? 0 : items.length - 1;
        paintList();
      }
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const items = filtered();
      const exact = items.find((i) => i.toLowerCase() === input.value.trim().toLowerCase());
      const pick = exact ?? items[activeIndex] ?? items[0];
      if (pick) {
        setValue(pick);
        close({ blur: true });
      }
    }
  });

  chevron.addEventListener("mousedown", (e) => {
    e.preventDefault();
    e.stopPropagation();
    toggle();
  });

  footer?.addEventListener("mousedown", (e) => {
    e.preventDefault();
    const next = !(opts.getShowAll?.() ?? showAll);
    showAll = next;
    opts.onShowAllChange?.(next);
    paintFooter();
  });

  const onDocPointer = (e: Event) => {
    if (!open) return;
    const t = e.target as Node | null;
    if (t && root.contains(t)) return;
    close();
  };
  document.addEventListener("pointerdown", onDocPointer, true);

  const onDocKey = (e: KeyboardEvent) => {
    if (!open) return;
    if (e.key === "Escape") {
      e.preventDefault();
      close({ blur: true });
    }
  };
  document.addEventListener("keydown", onDocKey, true);

  return {
    root,
    getValue: () => value,
    setValue,
    setOptions,
    setLabels: (labels) => {
      if (labels.showAll) showAllLabel = labels.showAll;
      if (labels.hideAll) hideAllLabel = labels.hideAll;
      if (labels.placeholder) input.placeholder = labels.placeholder;
      if (labels.toggle) {
        toggleLabel = labels.toggle;
        chevron.setAttribute("aria-label", toggleLabel);
      }
      if (labels.empty) emptyLabel = labels.empty;
      if (open) paintFooter();
    },
    close: () => close(),
    focus: () => input.focus(),
    destroy: () => {
      document.removeEventListener("pointerdown", onDocPointer, true);
      document.removeEventListener("keydown", onDocKey, true);
      root.remove();
    },
  };
}
