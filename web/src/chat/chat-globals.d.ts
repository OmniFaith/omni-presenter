interface ChatFile {
  id: string;
  name: string;
  size: number;
  mime: string;
}

interface ChatReaction {
  emoji: string;
  count: number;
  users: string[];
}

interface ChatMessage {
  id: string | number;
  channel_id?: string;
  recipient_id?: string;
  sender_id?: string;
  sender_name?: string;
  author?: string;
  author_id?: string;
  text?: string;
  content?: string;
  file?: ChatFile;
  timestamp: number | string;
  created_at?: string;
  reactions?: ChatReaction[];
  pinned?: boolean;
  kind?: string;
  song_id?: string | number;
  verse?: unknown;
  deleted?: boolean;
  shared?: any;
  title?: string;
}

interface ChatProfile {
  id: string;
  name: string;
  avatar?: string;
  avatar_url?: string;
  online?: boolean;
  role?: string;
}

interface ChatChannel {
  id: string;
  name: string;
  topic?: string;
  description?: string;
  tool?: string;
  unread?: number;
}

interface ChatNotifications {
  total: number;
  channels: Record<string, number>;
  private: Record<string, number>;
  recent: unknown[];
  latest_id: number;
}

interface ChatState {
  token: string;
  authenticated: boolean;
  profile: string;
  channel: string;
  peer: string | null;
  query: string;
  all: boolean;
  messages: ChatMessage[];
  channels: ChatChannel[];
  profiles: ChatProfile[];
  fingerprint: string;
  busy: boolean;
  loading: boolean;
  recorder: MediaRecorder | null;
  stream: MediaStream | null;
  chunks: Blob[];
  recordingTarget: { recipient_id?: string; channel_id?: string } | null;
  cancelRecording: boolean;
  recordTimer: ReturnType<typeof setInterval> | null;
  generation: number;
  history: boolean;
  hasOlder: boolean;
  pendingMessage: string | null;
  notifications: ChatNotifications;
  noticeCursor: unknown;
  webNotifications: boolean;
  emojiCategories: unknown[];
  emojiCategory: number;
  reactionTarget: unknown;
}

declare let state: ChatState;
declare let interfaceTexts: Record<string, string>;
declare const interfaceLanguage: string;
declare function $<T extends HTMLElement = HTMLElement>(id: string): T;
declare function node<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  text?: string,
  className?: string,
): HTMLElementTagNameMap[K];
declare var status: any;
declare function api<T = unknown>(
  path: string,
  options?: RequestInit & { json?: unknown },
): Promise<T>;
declare function icon(name: string): HTMLElement;
declare function paintIcons(root?: Document | HTMLElement): void;
declare function loadChatIcons(): Promise<void>;
declare function showLogin(): void;
declare function enter(): void;
declare function poll(): void;
declare function loadProfiles(): Promise<void>;
declare function selectRoom(channel: string | null, peer: string | null): void;
declare function composer(): void;
declare function formatSize(bytes: number): string;
declare function clock(seconds: number): string;
declare function target(): { recipient_id?: string; channel_id?: string };
declare function send(payload: Record<string, unknown>, destination?: unknown): Promise<unknown>;
declare function resourceHeader(kind: string, badge: string, title: string, subtitle?: string): HTMLElement;
declare function openMessage(id: string): void;
declare function profilePicture(container: HTMLElement, profile: ChatProfile | undefined, name: string): void;
declare function showLoginPhoto(): void;
declare function openEmojiPicker(messageId: string | number | null, rect: DOMRect): Promise<void>;
declare function fileElement(message: ChatMessage): HTMLElement;
declare let songTimer: any;
declare let songRequest: number;
declare function searchSongs(): void;

declare class ChatChannelList {
  constructor(options: any);
  channels(list: HTMLElement, items: any[], activeId: string | null): void;
  paint(): void;
  selected: Set<string>;
  anchor: string | null;
  menu: HTMLDivElement;
  dragIds: string[] | null;
}
