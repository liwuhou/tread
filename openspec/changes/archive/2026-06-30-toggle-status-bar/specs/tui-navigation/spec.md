## MODIFIED Requirements

### Requirement: Vim-style scroll navigation
系统 SHALL 支持 vim 风格的键盘导航。**新增**：`f` 键切换状态栏显隐。

#### Scenario: 按 f 隐藏状态栏（新增）
- **WHEN** 状态栏可见，用户按 `f`
- **THEN** 状态栏隐藏，正文区域占满全屏

#### Scenario: 再按 f 显示状态栏（新增）
- **WHEN** 状态栏已隐藏，用户按 `f`
- **THEN** 状态栏重新显示

#### Scenario: 帮助浮层显示 f 键说明（新增）
- **WHEN** 用户按 `?` 打开帮助
- **THEN** 帮助列表中显示 `f — 切换状态栏`
