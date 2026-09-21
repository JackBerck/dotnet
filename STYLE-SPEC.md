# STYLE-SPEC — Dotnet

## 1. Theme Philosophy
Classic Windows legacy style (Windows XP / Windows 7 basic). Function over form. Utilitarian, native, lightweight, zero visual overhead. No modern flat design.

## 2. Color Palette
Use system default colors strictly:
- **Background (Forms/Panels):** `SystemColors.Control` (Classic Gray)
- **Inputs/Lists Background:** `SystemColors.Window` (White)
- **Text:** `SystemColors.ControlText` (Black)
- **Borders/Lines:** `SystemColors.ControlDark` / `SystemColors.ControlLight`
- Avoid hardcoded custom HEX/RGB.

## 3. Typography
- **Primary Font:** `Tahoma` 8pt or `Microsoft Sans Serif` 8.25pt.
- **Console/Log:** `Consolas` 9pt or `Courier New` 9pt.
- Bold only for group headers or critical labels.

## 4. UI Controls
- **Standard WinForms Controls only:** Standard Buttons, CheckBoxes, DataGridView.
- **Borders:** `Fixed3D` for TextBoxes, RichTextBoxes, and ListBoxes.
- **Buttons:** System default `FlatStyle.Standard`. No `FlatStyle.Flat` with custom colors.
- **TabControl:** Default system rendering.
- No rounded corners, no custom padding tricks, no dark mode overrides. 

## 5. Layout
- Compact spacing. Standard margin: 3px to 6px.
- Use `GroupBox` for logically grouping related settings with visible border and title.
- Minimal padding.
