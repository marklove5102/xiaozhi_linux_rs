#!/usr/bin/env python3
import sys
import json
import time
import os

def main():
    # 从 stdin 读取参数
    try:
        input_data = sys.stdin.read()
        args = json.loads(input_data) if input_data else {}
    except:
        args = {}

    # 只接收文本内容参数，路径固定为脚本同目录的 task_output.log
    text = args.get("text", "这是后台任务写入的内容")
    
    # 获取脚本所在目录，强制保存到该目录下的 task_output.log
    script_dir = os.path.dirname(os.path.abspath(__file__))
    file_path = os.path.join(script_dir, "task_output.log")

    # 循环 10 次，每秒写入一次
    for i in range(1, 11):
        with open(file_path, "a", encoding="utf-8") as f:
            timestamp = time.strftime("%H:%M:%S", time.localtime())
            f.write(f"[{timestamp}] 进度 {i}/10: {text}\n")
        time.sleep(1)

    # 脚本的最终标准输出将作为“完成结果”回传给小智
    print(f"后台文件写入任务已圆满完成。文件保存在 {file_path}，共写入 10 行。")

if __name__ == "__main__":
    main()