export interface ShaderOptions {
  kind?: string;
  index?: number;
  colors?: [number, number, number][];
  speed?: number;
  scale?: number;
  intensity?: number;
  softness?: number;
  detail?: number;
  direction?: number;
  seed?: number;
  vignette?: number;
  grain?: number;
}

export interface PresentationTheme {
  id: string;
  name: string;
  top: string;
  bottom: string;
  colors?: string[];
  color: string;
  font: string;
  weight?: string | number;
  align?: CanvasTextAlign | string;
  case?: string;
  image?: boolean;
  shader?: boolean;
  shader_kind?: string;
  shader_index?: number;
  shader_options?: ShaderOptions;
  bg_type?: string;
  line_height?: number;
  letter_spacing?: number;
  text_case?: string;
}

