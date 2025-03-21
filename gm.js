// ==UserScript==
// @name         CQUST 学工考试自动化脚本
// @namespace    http://tampermonkey.net/
// @version      1.5
// @description  自动处理iframe内动态加载的考试内容（持续监听模式）
// @author       OpenTritium
// @match        http://xgbd.cqust.edu.cn:866/txxm/default.aspx*
// @grant        GM_xmlhttpRequest
// @require      https://cdn.jsdelivr.net/npm/xxhashjs@0.2.2/build/xxhash.min.js
// @run-at       document-end
// ==/UserScript==

(() => {
  "use strict";

  const TARGET_IFRAME_ID = "r_3_3";
  const CHECK_INTERVAL = 500;
  const SCRIPT_NAME = "CQUST_SMACK";
  const LIB = "https://github.moeyy.xyz/https://raw.githubusercontent.com/OpenTritium/CQUST_SMACK/refs/heads/master/solution_mapping.json";
  let isProcessing = false;

  // 检查iframe是否已加载完成
  const isIframeLoaded = (iframe) => {
    return !!iframe && (iframe.contentDocument?.readyState === 'complete');
  };

  // 执行核心操作
  const executeOperations = (iframe) => {
    if (isProcessing) return;
    isProcessing = true;

    const targetIframe = iframe || document.getElementById(TARGET_IFRAME_ID);
    if (!targetIframe) {
      console.warn(`[${SCRIPT_NAME}] 未找到目标 iframe`);
      isProcessing = false;
      return;
    }

    // 确保iframe内容已加载
    if (!isIframeLoaded(targetIframe)) {
      console.log(`[${SCRIPT_NAME}] 等待iframe加载完成`);
      setTimeout(() => executeOperations(targetIframe), CHECK_INTERVAL);
      return;
    }

    // 获取iframe文档
    const iframeDoc = targetIframe.contentDocument || targetIframe.contentWindow.document;

    // 请求题库数据
    GM_xmlhttpRequest({
      method: "GET",
      url: LIB,
      headers: { Accept: "application/json" },
      onload: (response) => {
        if (response.status >= 200 && response.status < 300) {
          const jsonData = JSON.parse(response.responseText);
          processQuestions(jsonData, iframeDoc);
        } else {
          console.error(`[${SCRIPT_NAME}] 请求失败，状态码：${response.status}`);
        }
        isProcessing = false;
      },
      onerror: (error) => {
        console.error(`[${SCRIPT_NAME}] 题库拉取失败：${error}`);
        isProcessing = false;
      }
    });
  };

  // 处理题目选择
  const processQuestions = (jsonData, iframeDoc) => {
    const selectAnswer = (topicType, t) => {
      const topicId = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_tm`;
      const topicSpan = iframeDoc.getElementById(topicId);
      if (!topicSpan) return;

      const topicText = topicSpan.textContent;
      const hash = window.XXH.h64(new TextEncoder().encode(topicText).buffer, 0).toString(10);
      const answer = jsonData[hash];

      if (!answer) {
        console.warn(`[${SCRIPT_NAME}] 未找到对应答案：${hash}`);
        return;
      }

      answer.forEach(c => {
        const inputId = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_xz_${c}`;
        const optionInput = iframeDoc.getElementById(inputId);
        if (optionInput) {
          optionInput.click();
          console.log(`[${SCRIPT_NAME}] 题目[${topicText}] 已选择答案：${c}`);
        }
      });
    };

    // 遍历所有题型
    [1, 2, 3].forEach((topicType, index) => {
      const count = [4, 20, 15][index]; // 单选/多选/判断题数量
      for (let t = 0; t < count; t++) {
        selectAnswer(topicType, t);
      }
    });

    console.log(`[${SCRIPT_NAME}] 自动答题完成`);
  };

  // 主监听函数
  const main = () => {
    // 监听整个文档变化（包括iframe内容变化）
    const docObserver = new MutationObserver(() => {
      const targetIframe = document.getElementById(TARGET_IFRAME_ID);
      if (targetIframe) {
        executeOperations(targetIframe);
      } else {
        console.log(`[${SCRIPT_NAME}] 检测到元素变化，但iframe不存在`);
      }
    });

    // 开始观察整个文档变化
    docObserver.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
      characterData: true
    });

    // 初始化时立即尝试执行
    executeOperations();
  };

  // 延迟执行确保DOM加载
  setTimeout(() => {
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get("dfldm") === "12") {
      main();
    }
  }, 2000);
})();