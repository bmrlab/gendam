资源管理器相关的 components, hooks 都放在这里

Explorer 的数据包含 4 部分
- UI 组件
- explorer hook: `useExplorerContext()` 包含数据（文件列表等）和设置，（由 `useExplorer()` 初始化 value)
- explorer view hook: `useExplorerViewContext()` 包含共享的 UI 组件, 比如右键菜单
- explorer store: `useExplorerStore()` 包含 UI 的临时状态, 比如菜单是否打开、是否正在重命名

被选中的文件 `selectedItems` 不是 UI 临时状态，应该和文件列表 `items` 一样放在 hook 中，作为一种数据，属于 **“过滤结果”** 数据。
