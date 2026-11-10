interface ChatChannelListOptions {
  state: () => ChatState;
  request: (path: string, options?: RequestInit & { json?: unknown }) => Promise<unknown>;
  refresh: () => void;
  open: (id: string) => void;
  create: () => void;
  report: (error: Error) => void;
  text: (key: string) => string;
}

interface ChannelItem {
  id: string;
  name: string;
  description?: string;
  tool?: string;
}

class ChatChannelList {
  options: ChatChannelListOptions;
  selected = new Set<string>();
  anchor: string | null = null;
  menu: HTMLDivElement;
  dragIds: string[] | null = null;

  constructor(options: ChatChannelListOptions) {
    this.options = options;
    this.menu = document.createElement("div");
    this.menu.className = "channel-context-menu";
    this.menu.setAttribute("role", "menu");
    this.menu.hidden = true;
    const mountMenu = () => {
      if (document.body && !this.menu.parentElement) {
        document.body.append(this.menu);
      }
    };
    if (document.body) {
      mountMenu();
    } else {
      document.addEventListener("DOMContentLoaded", mountMenu);
    }
    document.addEventListener("pointerdown", (event: PointerEvent) => {
      if (!this.menu.contains(event.target as Node)) this.menu.hidden = true;
    });
    document.addEventListener("keydown", (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        this.menu.hidden = true;
        this.selected.clear();
        this.paint();
      }
      if (!(event.target as HTMLElement | null)?.closest("#channels")) return;
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key.toLowerCase() === "a"
      ) {
        event.preventDefault();
        this.selected = new Set(
          this.options.state().channels.map((channel) => channel.id),
        );
        this.paint();
      }
      if (event.key === "Delete" && this.selected.size) {
        event.preventDefault();
        this.remove([...this.selected]);
      }
    });
  }

  bind(button: HTMLButtonElement, channel: ChannelItem): void {
    const order = this.options.state().channels.map((item) => item.id);
    this.selected = new Set(
      [...this.selected].filter((id) => order.includes(id)),
    );
    button.dataset.channelId = channel.id;
    button.classList.toggle(
      "channel-multiselected",
      this.selected.has(channel.id),
    );
    button.onclick = (event: MouseEvent) => {
      if (event.shiftKey && this.anchor && order.includes(this.anchor)) {
        if (!event.ctrlKey && !event.metaKey) this.selected.clear();
        const start = order.indexOf(this.anchor),
          end = order.indexOf(channel.id);
        order
          .slice(Math.min(start, end), Math.max(start, end) + 1)
          .forEach((id) => this.selected.add(id));
      } else if (event.ctrlKey || event.metaKey) {
        if (!this.selected.delete(channel.id)) this.selected.add(channel.id);
        this.anchor = channel.id;
      } else {
        this.selected = new Set([channel.id]);
        this.anchor = channel.id;
        this.options.open(channel.id);
      }
      this.paint();
    };
    button.oncontextmenu = (event: MouseEvent) => {
      event.preventDefault();
      if (!this.selected.has(channel.id)) this.selected = new Set([channel.id]);
      this.paint();
      this.showMenu(channel, event.clientX, event.clientY);
    };
    button.draggable = true;
    button.ondragstart = (event: DragEvent) => {
      if (!this.selected.has(channel.id)) this.selected = new Set([channel.id]);
      this.paint();
      const ids = order.filter((id) => this.selected.has(id));
      event.dataTransfer?.setData(
        "application/x-omni-channels",
        JSON.stringify(ids),
      );
      if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
      this.dragIds = ids;
      const preview = document.createElement("div");
      preview.className = "channel-drag-card";
      const title = document.createElement("strong");
      title.textContent = "# " + channel.name;
      const detail = document.createElement("span");
      detail.textContent =
        ids.length > 1
          ? this.options.text("drag.count").replace("{count}", String(ids.length))
          : this.options.text("drag.move");
      preview.append(title, detail);
      document.body.append(preview);
      event.dataTransfer?.setDragImage(preview, 30, 20);
      setTimeout(() => preview.remove(), 0);
    };
    button.ondragover = (event: DragEvent) => {
      if (
        !event.dataTransfer?.types.includes("application/x-omni-channels") ||
        this.dragIds?.includes(channel.id)
      )
        return;
      event.preventDefault();
      if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
      const after =
        event.clientY > button.getBoundingClientRect().top + button.offsetHeight / 2;
      button.classList.toggle("channel-drop-before", !after);
      button.classList.toggle("channel-drop-after", after);
    };
    button.ondragleave = () =>
      button.classList.remove("channel-drop-before", "channel-drop-after");
    button.ondragend = () => {
      this.dragIds = null;
      this.paint();
    };
    button.ondrop = async (event: DragEvent) => {
      event.preventDefault();
      const after = button.classList.contains("channel-drop-after");
      button.classList.remove("channel-drop-before", "channel-drop-after");
      try {
        const raw = event.dataTransfer?.getData("application/x-omni-channels");
        if (!raw) return;
        const ids = JSON.parse(raw);
        await this.options.request("/channels/reorder", {
          method: "POST",
          json: { ids, target: channel.id, after },
        });
        this.options.refresh();
      } catch (error) {
        this.options.report(error as Error);
      }
    };
  }

  paint(): void {
    document
      .querySelectorAll<HTMLElement>("#channels [data-channel-id]")
      .forEach((button) => {
        const id = button.dataset.channelId;
        button.classList.toggle(
          "channel-multiselected",
          !!(id && this.selected.has(id)),
        );
        button.classList.remove("channel-drop-before", "channel-drop-after");
      });
  }

  showMenu(channel: ChannelItem, x: number, y: number): void {
    this.menu.replaceChildren();
    const action = (key: string, callback: () => void | Promise<void>, danger = false) => {
      const button = document.createElement("button");
      button.textContent = this.options.text(key);
      button.setAttribute("role", "menuitem");
      if (danger) button.className = "danger";
      button.onclick = async () => {
        this.menu.hidden = true;
        try {
          await callback();
        } catch (error) {
          this.options.report(error as Error);
        }
      };
      this.menu.append(button);
    };
    const ids = this.options
      .state()
      .channels.map((item) => item.id)
      .filter((id) => this.selected.has(id));
    action("chat.open", () => this.options.open(channel.id));
    if (channel.id !== "general" && ids.length === 1)
      action("chat.edit", () => this.edit(channel));
    action("chat.read", async () => {
      await this.options.request("/channels/read", {
        method: "POST",
        json: { ids },
      });
      this.options.refresh();
    });
    action("chat.create", () => this.options.create());
    action("chat.select_all", () => {
      this.selected = new Set(
        this.options.state().channels.map((item) => item.id),
      );
      this.paint();
    });
    if (ids.some((id) => id !== "general"))
      action("chat.delete", () => this.remove(ids), true);
    this.menu.hidden = false;
    this.menu.style.left =
      Math.max(8, Math.min(x, window.innerWidth - this.menu.offsetWidth - 8)) +
      "px";
    this.menu.style.top =
      Math.max(
        8,
        Math.min(y, window.innerHeight - this.menu.offsetHeight - 8),
      ) + "px";
  }

  async remove(ids: string[]): Promise<void> {
    ids = ids.filter((id) => id !== "general");
    if (!ids.length) return;
    try {
      await this.options.request("/channels", {
        method: "DELETE",
        json: { ids },
      });
      this.selected.clear();
      if (ids.includes(this.options.state().channel))
        this.options.open("general");
      this.options.refresh();
    } catch (error) {
      this.options.report(error as Error);
    }
  }

  edit(channel: ChannelItem): void {
    const dialog = document.createElement("dialog");
    const form = document.createElement("form");
    const title = document.createElement("h2");
    title.textContent = this.options.text("chat.edit");
    const name = document.createElement("input");
    name.value = channel.name;
    name.required = true;
    name.maxLength = 80;
    name.setAttribute("aria-label", this.options.text("chat.channel_name"));
    const description = document.createElement("textarea");
    description.value = channel.description || "";
    description.maxLength = 500;
    description.setAttribute(
      "aria-label",
      this.options.text("chat.channel_description"),
    );
    const actions = document.createElement("div");
    actions.className = "actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = this.options.text("common.cancel");
    cancel.onclick = () => dialog.close();
    const save = document.createElement("button");
    save.className = "primary";
    save.textContent = this.options.text("common.save");
    actions.append(cancel, save);
    form.append(title, name, description, actions);
    dialog.append(form);
    document.body.append(dialog);
    dialog.onclose = () => dialog.remove();
    form.onsubmit = async (event: SubmitEvent) => {
      event.preventDefault();
      try {
        await this.options.request("/channels/update", {
          method: "POST",
          json: {
            id: channel.id,
            name: name.value,
            tool: channel.tool,
            description: description.value,
          },
        });
        dialog.close();
        this.options.refresh();
      } catch (error) {
        this.options.report(error as Error);
      }
    };
    dialog.showModal();
    name.focus();
  }
}

