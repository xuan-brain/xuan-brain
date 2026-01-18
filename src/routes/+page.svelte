<script lang="ts">
    // 主页面脚本
    let showSidebar = $state(true);
</script>

<main
    class="grid grid-cols-[15%_70%_15%] h-screen w-screen overflow-hidden bg-gray-200 dark:bg-gray-800"
>
    <!-- 左侧侧边栏 -->
    <aside
        class="bg-white dark:bg-gray-800 overflow-y-auto min-w-[150px] border-r border-gray-200 dark:border-gray-700 transition-all duration-300"
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
                        class="px-3 py-2.5 mb-1 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-gray-700 dark:text-gray-300"
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

    <!-- 中间主内容区 -->
    <main class="bg-gray-50 dark:bg-gray-900 overflow-y-auto flex flex-col">
        <div
            class="bg-white dark:bg-gray-800 p-5 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center"
        >
            <h1
                class="text-2xl font-semibold text-gray-900 dark:text-gray-100 m-0"
            >
                文献库
            </h1>
            <div class="flex gap-2.5">
                <button
                    class="px-4 py-2 text-sm font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded-lg border border-transparent cursor-pointer transition-all duration-200 shadow-[0_2px_4px_rgba(0,0,0,0.1)] hover:bg-blue-500 hover:text-white hover:-translate-y-0.5 hover:shadow-[0_4px_8px_rgba(0,0,0,0.15)] active:translate-y-0 active:shadow-[0_2px_4px_rgba(0,0,0,0.1)]"
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
        <div class="flex-1 p-5 overflow-y-auto">
            <!-- 文献列表将在这里渲染 -->
            <p
                class="text-gray-400 dark:text-gray-600 text-center italic mt-10"
            >
                暂无文献
            </p>
        </div>
    </main>

    <!-- 右侧侧边栏 -->
    <aside
        class="bg-white dark:bg-gray-800 overflow-y-auto min-w-[150px] border-l border-gray-200 dark:border-gray-700"
    >
        <div class="p-5">
            <h2
                class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4 pb-2 border-b-2 border-gray-200 dark:border-gray-700"
            >
                详情
            </h2>
            <div class="detail-panel">
                <p
                    class="text-gray-400 dark:text-gray-600 text-center italic mt-10"
                >
                    选择一篇文献查看详情
                </p>
            </div>
        </div>
    </aside>
</main>
