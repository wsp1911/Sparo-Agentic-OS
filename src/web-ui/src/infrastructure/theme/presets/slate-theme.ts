
import { ThemeConfig } from '../types';

/** Slate gray surfaces with Sparo OS ink-red accent (aligned with Light / Dark defaults). */
export const slateTheme: ThemeConfig = {
  
  id: 'slate',
  name: 'Slate',
  type: 'dark',
  description: 'Slate gray geometric theme — cool chrome with Sparo ink-red accent',
  version: '1.4.0',

  layout: {
    sceneViewportBorder: false,
  },
  
  colors: {
    background: {
      primary: '#14161a',
      secondary: '#22262c',
      tertiary: '#14161a',
      quaternary: '#2c3038',
      elevated: '#22262c',
      workbench: '#14161a',
      scene: '#22262c',
      tooltip: 'rgba(34, 38, 44, 0.96)',
    },
    
    text: {
      primary: '#eef0f3',
      secondary: '#c8ccd2',
      muted: '#9ea4ab',
      disabled: '#65696f',
    },
    
    
    accent: {
      50: 'rgba(183, 55, 47, 0.05)',
      100: 'rgba(183, 55, 47, 0.09)',
      200: 'rgba(183, 55, 47, 0.14)',
      300: 'rgba(183, 55, 47, 0.24)',
      400: 'rgba(183, 55, 47, 0.42)',
      500: '#B7372F',
      600: '#9E2F29',
      700: 'rgba(158, 47, 41, 0.88)',
      800: 'rgba(128, 36, 32, 0.94)',
    },
    
    
    purple: {
      50: 'rgba(184, 198, 255, 0.04)',
      100: 'rgba(184, 198, 255, 0.08)',
      200: 'rgba(184, 198, 255, 0.15)',
      300: 'rgba(184, 198, 255, 0.25)',
      400: 'rgba(184, 198, 255, 0.4)',
      500: '#b8c4ff',
      600: '#9dacf5',
      700: 'rgba(157, 172, 245, 0.8)',
      800: 'rgba(157, 172, 245, 0.9)',
    },
    
    semantic: {
      success: '#7fb899',       
      successBg: 'rgba(127, 184, 153, 0.1)',
      successBorder: 'rgba(127, 184, 153, 0.3)',
      
      warning: '#f59e0b',
      warningBg: 'rgba(245, 158, 11, 0.1)',
      warningBorder: 'rgba(245, 158, 11, 0.3)',
      
      error: '#B7372F',         
      errorBg: 'rgba(183, 55, 47, 0.12)',
      errorBorder: 'rgba(183, 55, 47, 0.35)',
      
      info: '#a8b0bd',
      infoBg: 'rgba(255, 255, 255, 0.07)',
      infoBorder: 'rgba(255, 255, 255, 0.2)',
      
      
      highlight: '#B7372F',
      highlightBg: 'rgba(183, 55, 47, 0.14)',
    },
    
    border: {
      subtle: 'rgba(255, 255, 255, 0.12)',    
      base: 'rgba(255, 255, 255, 0.18)',      
      medium: 'rgba(255, 255, 255, 0.24)',    
      strong: 'rgba(255, 255, 255, 0.32)',    
      prominent: 'rgba(255, 255, 255, 0.4)',  
    },
    
    element: {
      subtle: 'rgba(255, 255, 255, 0.05)',
      soft: 'rgba(255, 255, 255, 0.07)',
      base: 'rgba(255, 255, 255, 0.095)',
      medium: 'rgba(255, 255, 255, 0.125)',
      strong: 'rgba(255, 255, 255, 0.155)',
      elevated: 'rgba(255, 255, 255, 0.19)',
    },
    
    git: {
      branch: '#9ca6b8',
      branchBg: 'rgba(255, 255, 255, 0.06)',
      changes: 'rgb(245, 158, 11)',
      changesBg: 'rgba(245, 158, 11, 0.1)',
      added: 'rgb(127, 184, 153)',
      addedBg: 'rgba(127, 184, 153, 0.1)',
      deleted: '#B7372F',
      deletedBg: 'rgba(183, 55, 47, 0.12)',
      staged: 'rgb(127, 184, 153)',
      stagedBg: 'rgba(127, 184, 153, 0.1)',
    },
    
    scrollbar: {
      thumb: 'rgba(255, 255, 255, 0.15)',
      thumbHover: 'rgba(255, 255, 255, 0.28)',
    },
  },
  
  
  effects: {
    shadow: {
      xs: '0 1px 2px rgba(0, 0, 0, 0.85)',
      sm: '0 2px 4px rgba(0, 0, 0, 0.8)',
      base: '0 4px 8px rgba(0, 0, 0, 0.75)',
      lg: '0 8px 16px rgba(0, 0, 0, 0.7)',
      xl: '0 12px 24px rgba(0, 0, 0, 0.85)',
      '2xl': '0 16px 32px rgba(0, 0, 0, 0.9)',
    },
    
    glow: {
      blue: '0 12px 32px rgba(183, 55, 47, 0.14), 0 6px 16px rgba(183, 55, 47, 0.09), 0 3px 8px rgba(0, 0, 0, 0.2)',
      purple: '0 12px 32px rgba(184, 198, 255, 0.2), 0 6px 16px rgba(184, 198, 255, 0.12), 0 3px 8px rgba(0, 0, 0, 0.2)',
      mixed: '0 12px 32px rgba(255, 255, 255, 0.05), 0 6px 16px rgba(183, 55, 47, 0.08), 0 3px 8px rgba(0, 0, 0, 0.18)',
    },
    
    blur: {
      subtle: 'blur(4px) saturate(1.05) brightness(0.98)',
      base: 'blur(8px) saturate(1.08) brightness(0.98)',
      medium: 'blur(12px) saturate(1.12) brightness(0.97)',
      strong: 'blur(16px) saturate(1.15) brightness(0.97)',
      intense: 'blur(20px) saturate(1.18) brightness(0.96)',
    },
    
    radius: {
      sm: '4px',
      base: '6px',
      lg: '8px',
      xl: '12px',
      '2xl': '16px',
      full: '9999px',
    },
    
    spacing: {
      1: '4px',
      2: '8px',
      3: '12px',
      4: '16px',
      5: '20px',
      6: '24px',
      8: '32px',
      10: '40px',
      12: '48px',
      16: '64px',
    },
    
    opacity: {
      disabled: 0.5,
      hover: 0.75,
      focus: 0.85,
      overlay: 0.5,
    },
  },
  
  
  motion: {
    duration: {
      instant: '0.08s',
      fast: '0.12s',
      base: '0.25s',
      slow: '0.5s',
      lazy: '0.8s',
    },
    
    easing: {
      standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
      decelerate: 'cubic-bezier(0, 0, 0.2, 1)',
      accelerate: 'cubic-bezier(0.4, 0, 1, 1)',
      bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      smooth: 'cubic-bezier(0.4, 0, 0.2, 1)',
    },
  },
  
  
  typography: {
    font: {
      sans: "'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'SF Pro Display', Roboto, sans-serif",
      mono: "'FiraCode', 'JetBrains Mono', 'SF Mono', 'Consolas', 'Liberation Mono', monospace",
    },
    
    weight: {
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700,
    },
    
    size: {
      xs: '12px',
      sm: '13px',
      base: '14px',
      lg: '15px',
      xl: '16px',
      '2xl': '18px',
      '3xl': '22px',
      '4xl': '26px',
      '5xl': '32px',
    },
    
    lineHeight: {
      tight: 1.2,
      base: 1.5,
      relaxed: 1.6,
    },
  },
  
  
  components: {
    
    windowControls: {
      minimize: {
        dot: 'rgba(203, 213, 225, 0.42)',
        dotShadow: '0 0 4px rgba(0, 0, 0, 0.35)',
        hoverBg: 'rgba(255, 255, 255, 0.09)',
        hoverColor: '#e2e6eb',
        hoverBorder: 'rgba(255, 255, 255, 0.14)',
        hoverShadow: '0 2px 8px rgba(0, 0, 0, 0.22), inset 0 1px 0 rgba(255, 255, 255, 0.06)',
      },
      maximize: {
        dot: 'rgba(203, 213, 225, 0.42)',
        dotShadow: '0 0 4px rgba(0, 0, 0, 0.35)',
        hoverBg: 'rgba(255, 255, 255, 0.09)',
        hoverColor: '#e2e6eb',
        hoverBorder: 'rgba(255, 255, 255, 0.14)',
        hoverShadow: '0 2px 8px rgba(0, 0, 0, 0.22), inset 0 1px 0 rgba(255, 255, 255, 0.06)',
      },
      close: {
        dot: 'rgba(183, 55, 47, 0.5)',
        dotShadow: '0 0 4px rgba(183, 55, 47, 0.22)',
        hoverBg: 'rgba(183, 55, 47, 0.14)',
        hoverColor: '#B7372F',
        hoverBorder: 'rgba(183, 55, 47, 0.28)',
        hoverShadow: '0 2px 8px rgba(183, 55, 47, 0.16), inset 0 1px 0 rgba(255, 255, 255, 0.08)',
      },
      common: {
        defaultColor: 'rgba(232, 234, 236, 0.92)',
        defaultDot: 'rgba(198, 202, 208, 0.48)',
        disabledDot: 'rgba(168, 171, 176, 0.2)',
        flowGradient: 'linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.04), rgba(183, 55, 47, 0.1), rgba(255, 255, 255, 0.04), transparent)',
      },
    },
    
    button: {
      
      default: {
        background: 'rgba(255, 255, 255, 0.08)',
        color: '#a8b0bd',
        border: 'transparent',
        shadow: 'none',
      },
      hover: {
        background: 'rgba(255, 255, 255, 0.12)',
        color: '#dce0e6',
        border: 'transparent',
        shadow: 'none',
        transform: 'none',
      },
      active: {
        background: 'rgba(255, 255, 255, 0.1)',
        color: '#dce0e6',
        border: 'transparent',
        shadow: 'none',
        transform: 'none',
      },
      
      
      primary: {
        default: {
          background: '#B7372F',
          color: '#FFFFFF',
          border: 'transparent',
          shadow: '0 2px 8px rgba(183, 55, 47, 0.22)',
        },
        hover: {
          background: '#9E2F29',
          color: '#FFFFFF',
          border: 'transparent',
          shadow: '0 4px 14px rgba(183, 55, 47, 0.26)',
          transform: 'none',
        },
        active: {
          background: '#802420',
          color: '#FFFFFF',
          border: 'transparent',
          shadow: '0 2px 8px rgba(183, 55, 47, 0.22)',
          transform: 'none',
        },
      },
      
      
      ghost: {
        default: {
          background: 'transparent',
          color: '#a8b0bd',
          border: 'transparent',
          shadow: 'none',
        },
        hover: {
          background: 'rgba(255, 255, 255, 0.08)',
          color: '#dce0e6',
          border: 'transparent',
          shadow: 'none',
          transform: 'none',
        },
        active: {
          background: 'rgba(255, 255, 255, 0.06)',
          color: '#dce0e6',
          border: 'transparent',
          shadow: 'none',
          transform: 'none',
        },
      },
    },
  },
  
  
  monaco: {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment', foreground: '9ca2a9', fontStyle: 'italic' },
      { token: 'keyword', foreground: 'a8b4c4' },
      { token: 'string', foreground: '8fc8a9' },
      { token: 'number', foreground: 'b5c4fc' },
      { token: 'type', foreground: '9ca6b8' },
      { token: 'class', foreground: '9ca6b8' },
      { token: 'function', foreground: 'c5cad3' },
      { token: 'variable', foreground: 'c4c8ce' },
      { token: 'constant', foreground: 'b5c4fc' },
      { token: 'operator', foreground: 'a8b4c4' },
      { token: 'tag', foreground: '9ca6b8' },
      { token: 'attribute.name', foreground: 'c4c8ce' },
      { token: 'attribute.value', foreground: '8fc8a9' },
    ],
    colors: {
      background: '#1a1c1e',
      foreground: '#eef0f3',
      lineHighlight: '#22252a',
      selection: 'rgba(183, 55, 47, 0.26)',
      cursor: '#B7372F',
    },
  },
};
