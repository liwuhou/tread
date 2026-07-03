## ADDED Requirements

### Requirement: Full-screen terminal UI
系统 SHALL 启动时进入 alternate screen + raw mode，退出时完整恢复终端状态（包括光标显示）。

#### Scenario: 正常启动和退出
- **WHEN** 用户运行 `tread sample.md` 并按 `q` 退出
- **THEN** 终端恢复到启动前的状态，无残留内容，光标正常显示

#### Scenario: Ctrl+C 退出
- **WHEN** 用户按 `Ctrl+C`
- **THEN** 终端恢复到启动前的状态（与正常退出一致）

#### Scenario: Esc 退出
- **WHEN** 用户按 `Esc`
- **THEN** 终端恢复到启动前的状态

#### Scenario: 文件不存在
- **WHEN** 用户运行 `tread nonexistent.md`
- **THEN** 系统输出错误信息到 stderr，退出码非 0，不进入 TUI

### Requirement: Status bar
系统 SHALL 在终端底部显示一行状态栏，包含文件名、当前行号/总行数、滚动百分比。

#### Scenario: 状态栏内容
- **WHEN** 打开文件 `sample.md`，共 100 行，当前在第 25 行，终端高度 30
- **THEN** 状态栏显示 `" sample.md "` + `" 25/100 "` + `" XX% [?]帮助 [q]退出 "`

#### Scenario: 到达文件底部
- **WHEN** 滚动到文件末尾
- **THEN** 百分比显示为 100%

#### Scenario: 状态栏样式
- **WHEN** 状态栏渲染
- **THEN** 背景色为 DarkGray，文字为 White，文件名为 Bold

### Requirement: Help overlay
系统 SHALL 在用户按 `?` 时弹出居中浮层，显示所有可用快捷键。

#### Scenario: 显示帮助
- **WHEN** 用户在非帮助模式下按 `?`
- **THEN** 屏幕中央弹出半透明浮层，列出所有快捷键及其说明

#### Scenario: 关闭帮助
- **WHEN** 帮助浮层可见时按任意键
- **THEN** 浮层关闭，恢复正常阅读视图

#### Scenario: 帮助浮层内容
- **WHEN** 帮助浮层显示
- **THEN** 包含以下快捷键：j/k（上/下一行）、Ctrl+d/u（半页下/上）、Ctrl+f/b（整页下/上）、PgDn/PgUp（半页）、g/Home（顶部）、G/End（底部）、?（帮助）、q/Esc（退出）

### Requirement: Vim-style scroll navigation
系统 SHALL 支持 vim 风格的键盘导航。

#### Scenario: 单行滚动
- **WHEN** 用户按 `j` 或 `↓`
- **THEN** 内容向上滚动 1 行
- **WHEN** 用户按 `k` 或 `↑`
- **THEN** 内容向下滚动 1 行

#### Scenario: 半页滚动
- **WHEN** 用户按 `Ctrl+d`
- **THEN** 内容向下滚动 half_page 行（half_page = 终端内容高度 / 2）
- **WHEN** 用户按 `Ctrl+u`
- **THEN** 内容向上滚动 half_page 行

#### Scenario: 整页滚动
- **WHEN** 用户按 `Ctrl+f`
- **THEN** 内容向下滚动一整页（终端内容高度行）
- **WHEN** 用户按 `Ctrl+b`
- **THEN** 内容向上滚动一整页

#### Scenario: 跳转到顶部/底部
- **WHEN** 用户按 `g` 或 `Home`
- **THEN** 滚动到文件顶部（scroll = 0）
- **WHEN** 用户按 `G` 或 `End`
- **THEN** 滚动到文件底部（scroll = max_scroll）

#### Scenario: PageUp/PageDown
- **WHEN** 用户按 `PageDown`
- **THEN** 内容向下滚动 half_page 行
- **WHEN** 用户按 `PageUp`
- **THEN** 内容向上滚动 half_page 行

### Requirement: Scroll bounds
系统 SHALL 限制滚动范围，不可超出文件内容。

#### Scenario: 滚动到顶部边界
- **WHEN** 当前已在顶部（scroll = 0），用户按 `k`
- **THEN** scroll 保持为 0，不产生负值

#### Scenario: 滚动到底部边界
- **WHEN** 当前已在底部（scroll = max_scroll），用户按 `j`
- **THEN** scroll 保持为 max_scroll

#### Scenario: 内容不足一屏
- **WHEN** 文件行数少于终端高度
- **THEN** scroll 始终为 0，不可滚动

### Requirement: Terminal resize handling
系统 SHALL 在终端窗口大小变化时重新计算布局。

#### Scenario: 窗口缩小
- **WHEN** 用户在阅读过程中缩小终端窗口
- **THEN** 系统在下一帧重新计算行换行和可视区域，当前滚动位置尽量保持

#### Scenario: 窗口放大
- **WHEN** 用户在阅读过程中放大终端窗口
- **THEN** 系统重新换行并展示更多内容

### Requirement: Command-line interface
系统 SHALL 接受文件路径作为必需参数，可选接受起始行号。

#### Scenario: 正常打开文件
- **WHEN** 用户运行 `tread sample.md`
- **THEN** 系统打开文件并从第 1 行开始显示

#### Scenario: 指定起始行号
- **WHEN** 用户运行 `tread sample.md 50`
- **THEN** 系统打开文件并滚动到第 50 行

#### Scenario: 无参数运行
- **WHEN** 用户运行 `tread`（无参数）
- **THEN** 系统输出 usage 信息到 stderr，退出码为 1
### Requirement: Vim-style scroll navigation
系统 SHALL 支持 vim 风格的键盘导航。**新增**：Tab / Shift+Tab 在图片占位符之间跳转，Enter 打开当前焦点图片。
#### Scenario: Tab 跳到下一个图片（新增）
- **WHEN** 文档中有图片，用户按 Tab
- **THEN** 焦点跳到下一个图片占位符
#### Scenario: Shift+Tab 跳到上一个图片（新增）
- **WHEN** 文档中有图片，用户按 Shift+Tab
- **THEN** 焦点跳到上一个图片占位符
#### Scenario: Enter 打开焦点图片（新增）
- **WHEN** 焦点在某图片占位符上，用户按 Enter
- **THEN** 用系统默认图片浏览器打开该图片
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
