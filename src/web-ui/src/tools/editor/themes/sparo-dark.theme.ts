/**
 * Sparo OS built-in Monaco dark theme.
 * Syntax colors follow Night Owl–style readability; chrome accents align with Sparo UI (crimson / ink accent).
 */

import type { editor } from 'monaco-editor';

/** Matches default app dark preset (`presets/dark-theme`) for editor registration. */
export const SPARO_BUILTIN_DARK_MONACO_THEME_ID = 'sparo-dark' as const;

/**
 * Sparo OS Monaco dark palette for editors without a per-theme `monaco` block.
 * @see https://microsoft.github.io/monaco-editor/api/interfaces/monaco.editor.IStandaloneThemeData.html
 */
export const SparoOsDarkTheme: editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,

  rules: [
    // Comments
    { token: 'comment', foreground: '6A737D', fontStyle: 'italic' },
    { token: 'comment.line', foreground: '6A737D', fontStyle: 'italic' },
    { token: 'comment.block', foreground: '6A737D', fontStyle: 'italic' },
    { token: 'comment.doc', foreground: '6A737D', fontStyle: 'italic' },

    // Keywords
    { token: 'keyword', foreground: 'C792EA' },
    { token: 'keyword.control', foreground: 'C792EA' },
    { token: 'keyword.control.import', foreground: 'C792EA' },
    { token: 'keyword.control.export', foreground: 'C792EA' },
    { token: 'keyword.control.from', foreground: 'C792EA' },
    { token: 'keyword.operator', foreground: 'C792EA' },
    { token: 'keyword.operator.new', foreground: 'C792EA' },
    { token: 'keyword.other', foreground: 'C792EA' },

    // Strings
    { token: 'string', foreground: 'A5E844' },
    { token: 'string.quoted', foreground: 'A5E844' },
    { token: 'string.template', foreground: 'A5E844' },
    { token: 'string.regexp', foreground: 'A5E844' },

    // Numbers
    { token: 'number', foreground: 'F78C6C' },
    { token: 'number.hex', foreground: 'F78C6C' },
    { token: 'number.binary', foreground: 'F78C6C' },
    { token: 'number.octal', foreground: 'F78C6C' },
    { token: 'number.float', foreground: 'F78C6C' },

    // Functions and Methods
    { token: 'function', foreground: '7DCFFF' },
    { token: 'function.call', foreground: '7DCFFF' },
    { token: 'method', foreground: '7DCFFF' },
    { token: 'method.call', foreground: '7DCFFF' },
    { token: 'entity.name.function', foreground: '7DCFFF' },
    { token: 'support.function', foreground: '7DCFFF' },

    // Classes and Types
    { token: 'class', foreground: '4ECDC4' },
    { token: 'class.name', foreground: '4ECDC4' },
    { token: 'entity.name.class', foreground: '4ECDC4' },
    { token: 'entity.name.type.class', foreground: '4ECDC4' },
    { token: 'type', foreground: 'FFC777' },
    { token: 'type.identifier', foreground: 'FFC777' },
    { token: 'entity.name.type', foreground: 'FFC777' },
    { token: 'entity.other.inherited-class', foreground: '4ECDC4', fontStyle: 'italic' },
    { token: 'interface', foreground: '4ECDC4' },
    { token: 'entity.name.interface', foreground: '4ECDC4' },
    { token: 'enum', foreground: '73DACA' },
    { token: 'entity.name.enum', foreground: '73DACA' },
    { token: 'struct', foreground: '4ECDC4' },
    { token: 'entity.name.struct', foreground: '4ECDC4' },

    // Packages and Namespaces
    { token: 'namespace', foreground: '7AA2F7' },
    { token: 'entity.name.namespace', foreground: '7AA2F7' },
    { token: 'entity.name.package', foreground: '7AA2F7' },
    { token: 'entity.name.module', foreground: '7AA2F7' },
    { token: 'support.type.package', foreground: '7AA2F7' },

    // Variables
    { token: 'variable', foreground: '80D4FF' },
    { token: 'variable.name', foreground: '80D4FF' },
    { token: 'variable.parameter', foreground: 'E0E6F0' },
    { token: 'variable.other', foreground: '80D4FF' },
    { token: 'variable.language', foreground: 'C792EA', fontStyle: 'italic' },
    { token: 'variable.other.readwrite', foreground: '80D4FF' },
    { token: 'variable.other.property', foreground: '80D4FF' },
    { token: 'variable.other.constant', foreground: 'BB9AF7' },

    // Constants
    { token: 'constant', foreground: 'BB9AF7' },
    { token: 'constant.language', foreground: 'C792EA' },
    { token: 'constant.numeric', foreground: 'F78C6C' },
    { token: 'constant.character', foreground: 'A5E844' },

    // Operators and Punctuation
    { token: 'operator', foreground: 'C792EA' },
    { token: 'delimiter', foreground: 'E0E6F0' },
    { token: 'delimiter.bracket', foreground: '89DDFF' },
    { token: 'delimiter.parenthesis', foreground: '89DDFF' },
    { token: 'delimiter.square', foreground: '89DDFF' },

    // Tags (HTML/XML)
    { token: 'tag', foreground: '4ECDC4' },
    { token: 'tag.name', foreground: '4ECDC4' },
    { token: 'tag.attribute', foreground: 'C792EA', fontStyle: 'italic' },
    { token: 'tag.delimiter', foreground: '565F89' },

    // Special Tokens
    { token: 'annotation', foreground: 'FFC777' },
    { token: 'decorator', foreground: 'FFC777' },
    { token: 'attribute', foreground: 'C792EA', fontStyle: 'italic' },
    { token: 'meta', foreground: '7DCFFF' },
    { token: 'regexp', foreground: 'A5E844' },

    // Language-Specific: TypeScript/JavaScript
    { token: 'support.type.primitive', foreground: 'FFC777' },
    { token: 'support.type.builtin', foreground: 'FFC777' },
    { token: 'support.class', foreground: '4ECDC4' },
    { token: 'support.type.object', foreground: '4ECDC4' },
    { token: 'meta.import', foreground: 'C792EA' },
    { token: 'meta.export', foreground: 'C792EA' },

    // Language-Specific: Python
    { token: 'support.type.python', foreground: 'FFC777' },
    { token: 'meta.function.decorator.python', foreground: 'FFC777' },

    // Language-Specific: Java/C#
    { token: 'storage.modifier', foreground: 'C792EA', fontStyle: 'italic' },
    { token: 'storage.type', foreground: 'FFC777' },
    { token: 'meta.import.java', foreground: 'C792EA' },
    { token: 'storage.modifier.package.java', foreground: '7AA2F7' },
    { token: 'storage.modifier.import.java', foreground: 'C792EA' },

    // Language-Specific: C/C++
    { token: 'storage.type.built-in', foreground: 'FFC777' },
    { token: 'entity.name.type.typedef', foreground: 'FFC777' },
    { token: 'meta.preprocessor', foreground: 'C792EA', fontStyle: 'italic' },
    { token: 'keyword.control.directive', foreground: 'C792EA' },

    // Language-Specific: Rust
    { token: 'entity.name.type.rust', foreground: '4ECDC4' },
    { token: 'storage.type.rust', foreground: 'FFC777' },
    { token: 'support.type.primitive.rust', foreground: 'FFC777' },
    { token: 'entity.name.type.trait.rust', foreground: '4ECDC4' },

    // Language-Specific: Go
    { token: 'entity.name.package.go', foreground: '7AA2F7' },
    { token: 'storage.type.go', foreground: 'FFC777' },

    // Language-Specific: CSS
    { token: 'support.type.property-name', foreground: '80D4FF' },
    { token: 'entity.other.attribute-name', foreground: 'C792EA', fontStyle: 'italic' },

    // Language-Specific: Markdown
    { token: 'markup.heading', foreground: '7DCFFF' },
    { token: 'markup.bold', foreground: 'FFC777', fontStyle: 'bold' },
    { token: 'markup.italic', foreground: 'A5E844', fontStyle: 'italic' },
    { token: 'markup.underline', foreground: '80D4FF', fontStyle: 'underline' },
    { token: 'markup.quote', foreground: '6A737D', fontStyle: 'italic' },
    { token: 'markup.inline.raw', foreground: 'A5E844' },
    { token: 'markup.list', foreground: 'C792EA' },
    { token: 'markup.link', foreground: '7DCFFF', fontStyle: 'underline' },

    // Language-Specific: JSON
    { token: 'support.type.property-name.json', foreground: '80D4FF' },
    { token: 'string.key.json', foreground: '80D4FF' },
    { token: 'string.value.json', foreground: 'A5E844' },

    // Language-Specific: TOML
    { token: 'type.identifier.toml', foreground: 'FFC777' },
    { token: 'key.toml', foreground: '80D4FF' },
    { token: 'operator.toml', foreground: 'C792EA' },
    { token: 'string.toml', foreground: 'A5E844' },
    { token: 'string.quote.toml', foreground: 'A5E844' },
    { token: 'string.escape.toml', foreground: 'C792EA' },
    { token: 'string.invalid.toml', foreground: 'FF5370' },
    { token: 'number.toml', foreground: 'F78C6C' },
    { token: 'number.date.toml', foreground: 'F78C6C' },
    { token: 'number.float.toml', foreground: 'F78C6C' },
    { token: 'number.hex.toml', foreground: 'F78C6C' },
    { token: 'number.octal.toml', foreground: 'F78C6C' },
    { token: 'number.binary.toml', foreground: 'F78C6C' },
    { token: 'keyword.toml', foreground: 'C792EA' },
    { token: 'comment.toml', foreground: '6A737D', fontStyle: 'italic' },
    { token: 'delimiter.curly.toml', foreground: '89DDFF' },
    { token: 'delimiter.square.toml', foreground: '89DDFF' },
    { token: 'delimiter.bracket.toml', foreground: '89DDFF' },
    { token: 'delimiter.parenthesis.toml', foreground: '89DDFF' },
    { token: 'delimiter.comma.toml', foreground: 'E0E6F0' },
    { token: 'delimiter.dot.toml', foreground: 'E0E6F0' },

    // Semantic Tokens (LSP)
    { token: 'namespace', foreground: '7AA2F7' },
    { token: 'class', foreground: '4ECDC4' },
    { token: 'enum', foreground: '73DACA' },
    { token: 'interface', foreground: '4ECDC4' },
    { token: 'struct', foreground: '4ECDC4' },
    { token: 'typeParameter', foreground: 'FFC777' },
    { token: 'type', foreground: 'FFC777' },
    { token: 'parameter', foreground: 'E0E6F0' },
    { token: 'variable', foreground: '80D4FF' },
    { token: 'property', foreground: '80D4FF' },
    { token: 'enumMember', foreground: 'BB9AF7' },
    { token: 'event', foreground: 'FFC777' },
    { token: 'function', foreground: '7DCFFF' },
    { token: 'method', foreground: '7DCFFF' },
    { token: 'macro', foreground: '73DACA' },
    { token: 'keyword', foreground: 'C792EA' },
    { token: 'modifier', foreground: 'C792EA' },
    { token: 'comment', foreground: '6A737D' },
    { token: 'string', foreground: 'A5E844' },
    { token: 'number', foreground: 'F78C6C' },
    { token: 'regexp', foreground: 'A5E844' },
    { token: 'operator', foreground: 'C792EA' },
    { token: 'decorator', foreground: 'FFC777' },
    { token: 'label', foreground: 'C792EA' },
  ],

  colors: {
    // Global Border
    'focusBorder': '#00000000',
    'contrastBorder': '#00000000',

    // Editor Body
    'editor.background': '#121214',
    'editor.foreground': '#D6DEEB',

    // Line Numbers (crimson active / selection chrome)
    'editorLineNumber.foreground': '#707070',
    'editorLineNumber.activeForeground': '#B7372F',
    'editorLineNumber.dimmedForeground': '#454545',

    // Cursor and Selection
    'editorCursor.foreground': '#B7372F',
    'editorCursor.background': '#121214',
    'editor.selectionBackground': '#B7372F40',
    'editor.selectionForeground': '#FFFFFF',
    'editor.inactiveSelectionBackground': '#B7372F20',
    'editor.selectionHighlightBackground': '#B7372F30',
    'editor.selectionHighlightBorder': '#B7372F',

    // Current Line Highlight
    'editor.lineHighlightBackground': '#18181a',
    'editor.lineHighlightBorder': '#202024',

    // Find and Match
    'editor.findMatchBackground': '#B7372F',
    'editor.findMatchHighlightBackground': '#B7372F40',
    'editor.findRangeHighlightBackground': '#B7372F20',
    'editor.findMatchBorder': '#D9736A',
    'editor.findMatchHighlightBorder': '#B7372F80',

    // Word Highlight
    'editor.wordHighlightBackground': '#B7372F20',
    'editor.wordHighlightStrongBackground': '#B7372F40',
    'editor.wordHighlightBorder': '#B7372F60',
    'editor.wordHighlightStrongBorder': '#B7372F',

    // Code Highlight and Decorations
    'editor.hoverHighlightBackground': '#B7372F20',
    'editor.symbolHighlightBackground': '#B7372F20',
    'editor.symbolHighlightBorder': '#B7372F60',

    // Indent Guides and Rulers
    'editorIndentGuide.background': '#202024',
    'editorIndentGuide.activeBackground': '#B7372F60',
    'editorRuler.foreground': '#202024',

    // Bracket Matching
    'editorBracketMatch.background': '#B7372F30',
    'editorBracketMatch.border': '#B7372F',
    'editorBracketHighlight.foreground1': '#FFD700',
    'editorBracketHighlight.foreground2': '#B7372F',
    'editorBracketHighlight.foreground3': '#C792EA',
    'editorBracketHighlight.foreground4': '#4ECDC4',
    'editorBracketHighlight.foreground5': '#F78C6C',
    'editorBracketHighlight.foreground6': '#A5E844',

    // Suggest Widget
    'editorSuggestWidget.background': '#18181a',
    'editorSuggestWidget.border': '#B7372F',
    'editorSuggestWidget.foreground': '#E0E6F0',
    'editorSuggestWidget.highlightForeground': '#B7372F',
    'editorSuggestWidget.selectedBackground': '#B7372F30',
    'editorSuggestWidget.focusHighlightForeground': '#A5E844',

    // Hover Widget
    'editorHoverWidget.background': '#18181a',
    'editorHoverWidget.border': '#B7372F',
    'editorHoverWidget.foreground': '#E0E6F0',
    'editorHoverWidget.statusBarBackground': '#202024',

    // Inlay Hints
    'editorInlayHint.background': '#00000000',
    'editorInlayHint.foreground': '#6A737D',
    'editorInlayHint.typeForeground': '#6A737D',
    'editorInlayHint.parameterForeground': '#6A737D',

    // Errors and Warnings
    'editorError.foreground': '#FF5370',
    'editorWarning.foreground': '#FFCB6B',
    'editorInfo.foreground': '#8B93A8',
    'editorHint.foreground': '#6A737D',

    // Scrollbar
    'scrollbar.shadow': '#121214',
    'scrollbarSlider.background': '#B7372F40',
    'scrollbarSlider.hoverBackground': '#B7372F70',
    'scrollbarSlider.activeBackground': '#B7372FA0',

    // Minimap
    'minimap.background': '#121214',
    'minimap.selectionHighlight': '#B7372F40',
    'minimap.findMatchHighlight': '#B7372F',
    'minimap.errorHighlight': '#FF5370',
    'minimap.warningHighlight': '#FFCB6B',
    'minimapSlider.background': '#B7372F40',
    'minimapSlider.hoverBackground': '#B7372F70',
    'minimapSlider.activeBackground': '#B7372FA0',

    // Widget Borders
    'editorWidget.background': '#18181a',
    'editorWidget.border': '#B7372F40',
    'editorWidget.foreground': '#D6DEEB',
    'editorWidget.resizeBorder': '#B7372F60',

    // Code Lens
    'editorCodeLens.foreground': '#6A737D',

    // Links
    'editorLink.activeForeground': '#7DCFFF',

    // Whitespace
    'editorWhitespace.foreground': '#3A4A5A',

    // Overview Ruler
    'editorOverviewRuler.border': '#18181a',
    'editorOverviewRuler.background': '#121214',
    'editorOverviewRuler.currentContentForeground': '#B7372F80',
    'editorOverviewRuler.incomingContentForeground': '#7FDBCA80',
    'editorOverviewRuler.findMatchForeground': '#FFCB6B80',
    'editorOverviewRuler.rangeHighlightForeground': '#B7372F40',
    'editorOverviewRuler.selectionHighlightForeground': '#B7372F60',
    'editorOverviewRuler.wordHighlightForeground': '#C792EA60',
    'editorOverviewRuler.modifiedForeground': '#FFCB6B',
    'editorOverviewRuler.addedForeground': '#ADDB67',
    'editorOverviewRuler.deletedForeground': '#FF5370',
    'editorOverviewRuler.errorForeground': '#FF5370',
    'editorOverviewRuler.warningForeground': '#FFCB6B',
    'editorOverviewRuler.infoForeground': '#B7372F',

    // Diff Editor (GitHub Dark style)
    'diffEditor.insertedTextBackground': '#23863625',
    'diffEditor.insertedLineBackground': '#23863630',
    'diffEditor.insertedTextBorder': '#00000000',
    'diffEditorGutter.insertedLineBackground': '#23863638',

    'diffEditor.removedTextBackground': '#DA363325',
    'diffEditor.removedLineBackground': '#DA363330',
    'diffEditor.removedTextBorder': '#00000000',
    'diffEditorGutter.removedLineBackground': '#DA363338',

    'diffEditor.modifiedTextBackground': '#B7372F22',
    'diffEditor.modifiedLineBackground': '#B7372F28',

    'diffEditor.border': '#2A2D35',
    'diffEditor.diagonalFill': '#16181D',
    'diffEditor.unchangedRegionBackground': '#0D0D0F',
    'diffEditor.unchangedCodeBackground': '#0D0D0F',

    'diffEditorOverview.insertedForeground': '#3FB950',
    'diffEditorOverview.removedForeground': '#F85149',
  }
};

export const SparoOsDarkThemeMetadata = {
  id: SPARO_BUILTIN_DARK_MONACO_THEME_ID,
  label: 'Dark',
  description: 'Sparo OS built-in dark editor chrome with ink-red accents',
  version: '2.1.0',
};
