// ==UserScript==
// @name         CQUST 学工考试自动化脚本
// @namespace    http://tampermonkey.net/
// @version      1.6
// @description  自动处理iframe内动态加载的考试内容（单次拉取题库+持续监听）
// @author       OpenTritium
// @match        http://xgbd.cqust.edu.cn:866/txxm/default.aspx*
// @grant        GM_xmlhttpRequest
// @require      https://cdn.jsdelivr.net/npm/xxhashjs@0.2.2/build/xxhash.min.js
// @run-at       document-end
// ==/UserScript==

(() => {
  "use strict";

  const TARGET_IFRAME_ID = "r_3_3";
  const CHECK_INTERVAL = 50;
  const SCRIPT_NAME = "CQUST_SMACK";
  const LIB =
    "https://github.moeyy.xyz/https://raw.githubusercontent.com/OpenTritium/CQUST_SMACK/refs/heads/master/solution_mapping.json";
  let isProcessing = false;
  let jsonData = null;
  let dataPromise = null;

  // 提前初始化数据请求
  const initData = () => {
    if (!dataPromise) {
      dataPromise = new Promise((resolve, reject) => {
        GM_xmlhttpRequest({
          method: "GET",
          url: LIB,
          headers: { Accept: "application/json" },
          onload: (response) => {
            if (response.status >= 200 && response.status < 300) {
              jsonData = JSON.parse(response.responseText);
              resolve(jsonData);
            } else {
              reject(new Error(`请求失败，状态码：${response.status}`));
            }
          },
          onerror: (error) => reject(error),
        });
      });
    }
    return dataPromise;
  };

  const isIframeLoaded = (iframe) => {
    return !!iframe && iframe.contentDocument?.readyState === "complete";
  };

  const executeOperations = (iframe) => {
    if (isProcessing) return;
    isProcessing = true;

    const targetIframe = iframe || document.getElementById(TARGET_IFRAME_ID);
    if (!targetIframe) {
      console.warn(`[${SCRIPT_NAME}] 未找到目标 iframe`);
      isProcessing = false;
      return;
    }

    if (!isIframeLoaded(targetIframe)) {
      console.log(`[${SCRIPT_NAME}] 等待iframe加载完成`);
      setTimeout(() => executeOperations(targetIframe), CHECK_INTERVAL);
      return;
    }

    const iframeDoc =
      targetIframe.contentDocument || targetIframe.contentWindow.document;

    dataPromise
      .then((data) => {
        processQuestions(data, iframeDoc);
        isProcessing = false;
      })
      .catch((error) => {
        console.error(`[${SCRIPT_NAME}] 题库拉取失败：${error}`);
        isProcessing = false;
      });
  };

  const processQuestions = (data, iframeDoc) => {
    const selectAnswer = (topicType, t) => {
      const topicId = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_tm`;
      const topicSpan = iframeDoc.getElementById(topicId);
      if (!topicSpan) return;

      const topicText = topicSpan.textContent;
      const hash = window.XXH.h64(
        new TextEncoder().encode(topicText).buffer,
        0
      ).toString(10);
      const answer = data[hash];

      if (!answer) {
        console.warn(`[${SCRIPT_NAME}] 未找到对应答案：${hash}`);
        return;
      }

      answer.forEach((c) => {
        const inputId = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_xz_${c}`;
        const optionInput = iframeDoc.getElementById(inputId);
        if (optionInput) {
          forceClick(optionInput);
          console.log(`[${SCRIPT_NAME}] 题目[${topicText}] 已选择答案：${c}`);
        }
      });
    };

    [1, 2, 3].forEach((topicType, index) => {
      const count = [4, 20, 15][index];
      for (let t = 0; t < count; t++) {
        selectAnswer(topicType, t);
      }
    });

    console.log(`[${SCRIPT_NAME}] 自动答题完成`);
  };

  const forceClick = (input) => {
    input.click();
    if (input.type === "checkbox" && !input.checked) {
      input.checked = true;
      dispatchEvent(new Event("change"));
    }
  };

  const main = () => {
    initData().catch((error) =>
      console.error(`[${SCRIPT_NAME}] 初始化失败: ${error}`)
    );

    const docObserver = new MutationObserver(() => {
      let targetIframe = document.getElementById(TARGET_IFRAME_ID);
      if (targetIframe) {
        executeOperations(targetIframe);
      }
    });

    docObserver.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
      characterData: true,
    });
  };

  setTimeout(() => {
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get("dfldm") === "12") {
      main();
    }
  }, 500);
})();
