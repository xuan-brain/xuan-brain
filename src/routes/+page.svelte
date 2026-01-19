<script lang="ts">
  // 从 localStorage 加载保存的列宽，如果没有则使用默认值
  const STORAGE_KEY = "xuan-brain-layout-widths";

  // 最小宽度（百分比）
  const MIN_WIDTH_PERCENT = 10;
  const MAX_WIDTH_PERCENT = 40;

  function loadWidths(): { left: number; right: number } {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const widths = JSON.parse(saved);
        return {
          left: Math.max(
            MIN_WIDTH_PERCENT,
            Math.min(MAX_WIDTH_PERCENT, widths.left),
          ),
          right: Math.max(
            MIN_WIDTH_PERCENT,
            Math.min(MAX_WIDTH_PERCENT, widths.right),
          ),
        };
      }
    } catch (e) {
      console.error("Failed to load layout widths:", e);
    }
    return { left: 15, right: 15 };
  }

  const savedWidths = loadWidths();

  // 列宽状态（百分比）
  let leftWidth = $state(savedWidths.left);
  let rightWidth = $state(savedWidths.right);
  let middleWidth = $derived(100 - leftWidth - rightWidth);

  // 拖动状态
  let isDraggingLeft = $state(false);
  let isDraggingRight = $state(false);
  let startX = $state(0);
  let startLeftWidth = $state(0);
  let startRightWidth = $state(0);

  // 监听列宽变化并保存到 localStorage
  $effect(() => {
    // 只在列宽实际变化时保存
    try {
      const widthsToSave = JSON.stringify({
        left: leftWidth,
        right: rightWidth,
      });
      localStorage.setItem(STORAGE_KEY, widthsToSave);
    } catch (e) {
      console.error("Failed to save layout widths:", e);
    }
  });

  // 状态栏状态
  let currentTime = $state(new Date());
  let documentCount = $state(0);
  let syncStatus = $state("已同步");
  let isSyncing = $state(false);
  let searchStatus = $state("就绪");
  let memoryUsage = $state("0 MB");

  // 每秒更新时间和内存使用
  setInterval(() => {
    currentTime = new Date();
    // 模拟内存使用（实际项目中可以使用 performance.memory 或 Tauri API）
    const memory = Math.floor(Math.random() * 100 + 50);
    memoryUsage = `${memory} MB`;
  }, 1000);

  // 左侧拖动手柄
  function handleLeftResizerMouseDown(event: MouseEvent) {
    isDraggingLeft = true;
    startX = event.clientX;
    startLeftWidth = leftWidth;

    // 添加全局事件监听
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    // 防止选中文本
    event.preventDefault();
  }

  // 右侧拖动手柄
  function handleRightResizerMouseDown(event: MouseEvent) {
    isDraggingRight = true;
    startX = event.clientX;
    startRightWidth = rightWidth;

    // 添加全局事件监听
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    // 防止选中文本
    event.preventDefault();
  }

  // 鼠标移动
  function handleMouseMove(event: MouseEvent) {
    if (!isDraggingLeft && !isDraggingRight) return;

    const containerWidth = window.innerWidth;
    const deltaX = event.clientX - startX;
    const deltaPercent = (deltaX / containerWidth) * 100;

    if (isDraggingLeft) {
      const newWidth = startLeftWidth + deltaPercent;
      leftWidth = Math.max(
        MIN_WIDTH_PERCENT,
        Math.min(MAX_WIDTH_PERCENT, newWidth),
      );
    }

    if (isDraggingRight) {
      const newWidth = startRightWidth - deltaPercent;
      rightWidth = Math.max(
        MIN_WIDTH_PERCENT,
        Math.min(MAX_WIDTH_PERCENT, newWidth),
      );
    }
  }

  // 鼠标释放
  function handleMouseUp() {
    isDraggingLeft = false;
    isDraggingRight = false;

    // 移除全局事件监听
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }
</script>

<svelte:window />

<div
  class="h-[calc(100vh-36px)] w-screen overflow-hidden bg-gray-200 dark:bg-gray-800 flex"
>
  <!-- 左侧侧边栏 -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-y-auto min-w-37.5 border-r border-gray-200 dark:border-gray-700 shrink-0"
    style="width: {leftWidth}%;"
  >
    <div class="p-5">
      <h2
        class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4 pb-2 border-b-2 border-gray-200 dark:border-gray-700"
      >
        导航
      </h2>
      <nav>
        <ul class="list-none p-0 m-0">
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            文献库
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            分类
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100
 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            标签
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            收藏
          </li>
          <li
            class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
          >
            回收站
          </li>
        </ul>
      </nav>
    </div>
  </aside>

  <!-- 左侧拖动手柄 -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    aria-valuenow={leftWidth}
    aria-valuemin={MIN_WIDTH_PERCENT}
    aria-valuemax={MAX_WIDTH_PERCENT}
    aria-label="调整左侧栏宽度"
    class="w-0.5 bg-gray-300 dark:bg-gray-600 hover:bg-blue-500 dark:hover:bg-blue-500 cursor-col-resize shrink-0 transition-colors duration-150 z-10"
    class:bg-blue-500={isDraggingLeft}
    class:dark:bg-blue-500={isDraggingLeft}
    onmousedown={handleLeftResizerMouseDown}
  ></div>

  <!-- 中间主内容区 -->
  <main
    class="bg-gray-50 dark:bg-gray-900 overflow-y-auto flex flex-col flex-1 min-w-0"
  >
    <div
      class="bg-white dark:bg-gray-800 p-5 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center shrink-0"
    >
      <h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 m-0">
        文献库
      </h1>
      <div class="flex gap-2.5">
        <button
          class="px-4 py-2 text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg border border-transparent cursor-pointer transition-all duration-200 shadow-[0_2px_4px_rgba(0,0,0,0.1)] hover:bg-blue-500 hover:text-white hover:-translate-y-0.5 hover:shadow-[0_4px_8px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_4px_rgba(0,0
,0,0.1)]"
        >
          导入文献
        </button>
        <button
          class="px-4 py-2 text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg border border-transparent cursor-pointer transition-all duration-200 shadow-[0_2px_4px_rgba(0,0,0,0.1)] hover:bg-blue-500 hover:text-white hover:-translate-y-0.5 hover:shadow-[0_4px_8px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_4px_rgba(0,0,0,0.1)]"
        >
          搜索
        </button>
      </div>
    </div>
    <div class="flex-1 p-5 overflow-y-auto min-h-0">
      <!-- 文献列表将在这里渲染 -->
      <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
        暂无文献
      </p>
    </div>
  </main>

  <!-- 右侧拖动手柄 -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="separator"
    aria-orientation="vertical"
    aria-valuenow={rightWidth}
    aria-valuemin={MIN_WIDTH_PERCENT}
    aria-valuemax={MAX_WIDTH_PERCENT}
    aria-label="调整右侧栏宽度"
    class="w-0.5 bg-gray-300 dark:bg-gray-600 hover:bg-blue-500 dark:hover:bg-blue-500 cursor-col-resize shrink-0 transition-colors duration-150 z-10"
    class:bg-blue-500={isDraggingRight}
    class:dark:bg-blue-500={isDraggingRight}
    onmousedown={handleRightResizerMouseDown}
  ></div>

  <!-- 右侧侧边栏 -->
  <aside
    class="bg-white dark:bg-gray-800 overflow-y-auto min-w-37.5 border-l border-gray-200 dark:border-gray-700 shrink-0"
    style="width: {rightWidth}%;"
  >
    <div class="p-5">
      <h2
        class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4 pb-2 border-b-2 border-gray-200 dark:border-gray-700"
      >
        详情
      </h2>
      <div class="detail-panel">
        <p class="text-gray-400 dark:text-gray-600 text-center italic mt-10">
          选择一篇文献查看详情
        </p>
      </div>
    </div>
  </aside>
</div>

<!-- 状态栏 -->
<footer
  class="h-9 bg-gray-800 dark:bg-gray-950 border-t border-gray-700 dark:border-gray-800 flex items-center justify-between px-4 text-xs text-gray-300 dark:text-gray-400 select-none"
>
  <div class="flex items-center gap-4">
    <button
      class="flex items-center gap-1.5 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      class:animate-pulse={isSyncing}
      disabled={isSyncing}
      onclick={() => {
        isSyncing = true;
        syncStatus = "同步中...";
        setTimeout(() => {
          isSyncing = false;
          syncStatus = "已同步";
        }, 2000);
      }}
    >
      <span
        class="w-2 h-2 rounded-full"
        class:bg-green-500={syncStatus === "已同步"}
        class:bg-yellow-500={syncStatus === "同步中..."}
        class:bg-gray-500={syncStatus === "未同步"}
      ></span>
      {syncStatus}
    </button>
    <span class="flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          d="M9 4.804A7.968 7.968 0 005.5 4c-1.255 0-2.443.29-3.5.804v10A7.969 7.969 0 015.5 14c1.669 0 3.218.51 4.5 1.385A7.962 7.962 0 0114.5 14c1.255 0 2.443.29 3.5.804v-10A7.968 7.968 0 0014.5 4c-1.255 0-2.443.29-3.5.804V12a1 1 0 11-2 0V4.804z"
        />
      </svg>
      文献数量: {documentCount}
    </span>
    <span class="hidden md:flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
          clip-rule="evenodd"
        />
      </svg>
      {searchStatus}
    </span>
    <span class="hidden lg:flex items-center gap-1">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="w-3.5 h-3.5"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z"
          clip-rule="evenodd"
        />
      </svg>
      {memoryUsage}
    </span>
  </div>
  <div class="flex items-center gap-4">
    <span class="hidden sm:inline">版本 0.1.0</span>
    <span class="font-mono bg-gray-700 dark:bg-gray-900 px-2 py-0.5 rounded">
      {currentTime.toLocaleTimeString()}
    </span>
  </div>
</footer>

<style>
  /* 防止拖动时选中文字 */
  :global(.cursor-col-resize) {
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
  }
</style>
