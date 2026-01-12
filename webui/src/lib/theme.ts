/**
 * Copyright 2025 Meta-Hybrid Mount Authors
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { 
  argbFromHex, 
  hexFromArgb, 
  Hct,
  SchemeTonalSpot,
  SchemeMonochrome,
  SchemeFidelity,
  SchemeVibrant,
  SchemeExpressive,
  SchemeNeutral,
  SchemeFruitSalad,
  SchemeRainbow,
  DynamicScheme
} from '@material/material-color-utilities';

export type ThemeStyle = 'TONAL_SPOT' | 'MONOCHROME' | 'VIBRANT' | 'EXPRESSIVE' | 'NEUTRAL' | 'FRUIT_SALAD' | 'RAINBOW' | 'FIDELITY';

export const Monet = {
  apply: (seedHex: string | null, isDark: boolean, style: ThemeStyle = 'TONAL_SPOT') => {
    if (!seedHex) return;
    
    let seedArgb;
    try {
        seedArgb = argbFromHex(seedHex);
    } catch (e) {
        return;
    }
    
    const sourceColor = Hct.fromInt(seedArgb);
    let scheme: DynamicScheme;

    switch (style) {
        case 'MONOCHROME':
            scheme = new SchemeMonochrome(sourceColor, isDark, 0.0);
            break;
        case 'VIBRANT':
            scheme = new SchemeVibrant(sourceColor, isDark, 0.0);
            break;
        case 'EXPRESSIVE':
            scheme = new SchemeExpressive(sourceColor, isDark, 0.0);
            break;
        case 'NEUTRAL':
            scheme = new SchemeNeutral(sourceColor, isDark, 0.0);
            break;
        case 'FRUIT_SALAD':
            scheme = new SchemeFruitSalad(sourceColor, isDark, 0.0);
            break;
         case 'RAINBOW':
            scheme = new SchemeRainbow(sourceColor, isDark, 0.0);
            break;
        case 'FIDELITY':
            scheme = new SchemeFidelity(sourceColor, isDark, 0.0);
            break;
        case 'TONAL_SPOT':
        default:
            scheme = new SchemeTonalSpot(sourceColor, isDark, 0.0);
            break;
    }

    const tokens: Record<string, number> = {
      '--md-sys-color-primary': scheme.primary,
      '--md-sys-color-on-primary': scheme.onPrimary,
      '--md-sys-color-primary-container': scheme.primaryContainer,
      '--md-sys-color-on-primary-container': scheme.onPrimaryContainer,
      '--md-sys-color-secondary': scheme.secondary,
      '--md-sys-color-on-secondary': scheme.onSecondary,
      '--md-sys-color-secondary-container': scheme.secondaryContainer,
      '--md-sys-color-on-secondary-container': scheme.onSecondaryContainer,
      '--md-sys-color-tertiary': scheme.tertiary,
      '--md-sys-color-on-tertiary': scheme.onTertiary,
      '--md-sys-color-tertiary-container': scheme.tertiaryContainer,
      '--md-sys-color-on-tertiary-container': scheme.onTertiaryContainer,
      '--md-sys-color-error': scheme.error,
      '--md-sys-color-on-error': scheme.onError,
      '--md-sys-color-error-container': scheme.errorContainer,
      '--md-sys-color-on-error-container': scheme.onErrorContainer,
      '--md-sys-color-background': scheme.background,
      '--md-sys-color-on-background': scheme.onBackground,
      '--md-sys-color-surface': scheme.surface,
      '--md-sys-color-on-surface': scheme.onSurface,
      '--md-sys-color-surface-variant': scheme.surfaceVariant,
      '--md-sys-color-on-surface-variant': scheme.onSurfaceVariant,
      '--md-sys-color-outline': scheme.outline,
      '--md-sys-color-outline-variant': scheme.outlineVariant,
      '--md-sys-color-surface-container-low': scheme.surfaceContainerLow,
      '--md-sys-color-surface-container': scheme.surfaceContainer,
      '--md-sys-color-surface-container-high': scheme.surfaceContainerHigh,
      '--md-sys-color-surface-container-highest': scheme.surfaceContainerHighest,
      '--md-sys-color-inverse-surface': scheme.inverseSurface,
      '--md-sys-color-inverse-on-surface': scheme.inverseOnSurface,
      '--md-sys-color-inverse-primary': scheme.inversePrimary,
      '--md-sys-color-shadow': scheme.shadow,
    };
    
    const root = document.documentElement.style;
    for (const [key, value] of Object.entries(tokens)) {
      root.setProperty(key, hexFromArgb(value));
    }
  }
};