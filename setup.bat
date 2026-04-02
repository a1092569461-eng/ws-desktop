@echo off
echo ========================================
echo WebSocket 消息监控桌面版 - 构建脚本
echo ========================================
echo.

echo [1/3] 检查 Rust 环境...
rustc --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未安装 Rust，请先安装: https://rustup.rs
    pause
    exit /b 1
)
echo Rust 已安装

echo.
echo [2/3] 检查 Node.js 环境...
node --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未安装 Node.js，请先安装: https://nodejs.org
    pause
    exit /b 1
)
echo Node.js 已安装

echo.
echo [3/3] 安装依赖...
call npm install

echo.
echo ========================================
echo 环境准备完成！
echo.
echo 可用命令:
echo   npm run dev    - 开发模式运行
echo   npm run build  - 构建生产版本
echo ========================================
pause
