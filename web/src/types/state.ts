import type { PresentationTheme } from "./theme.js";

export interface PlaylistItem {
  id: string | number;
  title: string;
  subtitle?: string;
  kind?: string;
  played?: boolean;
  selected?: boolean;
}

export interface QuickBarItem {
  id: string | number;
  title: string;
  kind?: string;
}

export interface ServiceDateItem {
  date: string;
  label: string;
  items: number;
  active: boolean;
}

export interface ServiceItem {
  id: string;
  name: string;
}

export interface BibleContext {
  book?: string;
  chapter?: number;
  start?: number;
  end?: number;
}

export interface DeckItem {
  title?: string;
  subtitle?: string;
  slides?: string[];
  index?: number;
  raw?: string;
  song_id?: number | null;
  design?: string | null;
  theme?: PresentationTheme;
}

export interface PlayerState {
  title: string;
  playing: boolean;
  position: number;
  duration: number;
  volume: number;
}

export interface TimerItem {
  name: string;
  text: string;
  running: boolean;
}

export interface ScreenItem {
  id: string;
  name: string;
  slug: string;
}

export interface ControlState {
  ready?: boolean;
  presenting?: boolean;
  hidden?: boolean;
  dark?: boolean;
  language?: string;
  catalog_revision?: string;
  bible_version?: string;
  layout?: string;
  bible_layout?: string;
  scale?: number;
  lyrics_cards?: boolean;
  show_dock?: boolean;
  dock_background?: boolean;
  progress?: number;
  aspect?: string;
  idle_theme?: PresentationTheme;
  bible_context?: BibleContext;
  operator?: string;
  chat_enabled?: boolean;
  service?: string;
  service_date?: string;
  service_dates?: ServiceDateItem[];
  services?: ServiceItem[];
  preview?: DeckItem;
  live?: DeckItem;
  playlist?: PlaylistItem[];
  quick?: QuickBarItem[];
  themes?: PresentationTheme[];
  player?: PlayerState | null;
  timers?: TimerItem[];
  questions?: boolean;
  overlay?: string | null;
  screens?: ScreenItem[];
}

