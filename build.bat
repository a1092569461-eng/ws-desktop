@echo off
echo 构建生产版本...
call npm run build
echo.
echo 构建完成！可执行文件位于: src-tauri/target/release/
pause
