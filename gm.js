// ==UserScript==
// @name         学工考试自动化脚本
// @namespace    http://tampermonkey.net/
// @version      1.2
// @description  自动处理iframe内动态加载的考试内容
// @author       YourName
// @match        http://xgbd.cqust.edu.cn:866/txxm/default.aspx*
// @grant        none
// @require      https://cdn.jsdelivr.net/npm/murmur-128@1.0.0/index.js
// @run-at       document-end
// ==/UserScript==

(() => {
  "use strict";

  const TARGET_IFRAME_ID = "r_3_3";
  const WAIT_TIMEOUT = 15000;
  const CHECK_INTERVAL = 500;
  const SCRIPT_NAME = "CQUST_SMACK";
  let isOperationDone = false;

  const executeOperations = () => {
    if (isOperationDone) return;
    const targetIframe = document.getElementById(TARGET_IFRAME_ID);
    if (!targetIframe) {
      console.warn(`[${SCRIPT_NAME}] 未找到目标 iframe`);
      return;
    }

    const active_click = (jsonData, iframeDoc) => {
      const select = (jsonData, iframeDoc, topicType, t) => {
        let topic = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_tm`;
        const topicSpan = iframeDoc.getElementById(topic);
        let topic_text = topicSpan.textContent;
        let hash = window.murmur128(topic_text);
        let cc = jsonData[hash];
        if (cc == undefined) return;
        console.log(`[${SCRIPT_NAME}] 题目：${topicSpan.textContent}`);
        for (c in cc) {
          let input = `Mydatalist__ctl0_Mydatalist${topicType}__ctl${t}_xz_${c}`;
          const OptionInput = iframeDoc.getElementById(input);
          if (OptionInput && topicSpan) {
            OptionInput.click();
            console.log(`[${SCRIPT_NAME}] 已点击`);
          } else {
            setTimeout(iframeLoader, CHECK_INTERVAL);
          }
        }
      };
      for (t = 0; t != 4; ++t) {
        const multiChoice = 1;
        select(jsonData, iframeDoc, multiChoice, t);
      }
      for (t = 0; t != 20; ++t) {
        const multiSelect = 2;
        select(jsonData, iframeDoc, multiSelect, t);
      }
      for (t = 0; t != 15; ++t) {
        const trueOrFalse = 3;
        select(jsonData, iframeDoc, trueOrFalse, t);
      }
      isOperationDone = true;
    };

    const iframeLoader = () => {
      try {
        const iframeDoc =
          targetIframe.contentDocument || targetIframe.contentWindow.document;
        GM_xmlhttpRequest({
          method: "GET",
          url: "https://api.example.com/data.json",
          headers: {
            Accept: "application/json",
          },
          onload: (response) => {
            try {
              if (response.status >= 200 && response.status < 300) {
                const jsonData = JSON.parse(response.responseText);
                active_click(jsonData, iframeDoc);
              } else {
                console.error(
                  `[${SCRIPT_NAME}] 请求失败，状态码：${response.status}`
                );
              }
            } catch (error) {
              console.error(`[${SCRIPT_NAME}] 题库解析失败：${error}`);
            }
          },
          onerror: function (error) {
            console.error(`[${SCRIPT_NAME}] 题库拉取失败：${error}`);
          },
        });
      } catch (e) {
        console.error(`[${SCRIPT_NAME}] 跨域访问被阻止，请确认网站权限`);
      }
    };

    if (targetIframe.contentDocument.readyState === "complete") {
      iframeLoader();
    } else {
      targetIframe.onload = iframeLoader;
    }
  };

  const urlParams = new URLSearchParams(window.location.search);
  if (urlParams.get("dfldm") !== "12") return;

  const main = () => {
    executeOperations();
    const observer = new MutationObserver(() => {
      if (!isOperationDone) {
        executeOperations();
      }
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true,
    });

    setTimeout(() => {
      observer.disconnect();
      if (!isOperationDone) {
        console.warn(
          `[${SCRIPT_NAME}] 操作未完成，可能原因：
                1. 元素ID已变更
                2. 跨域限制未解除
                3. 内容加载超时`
        );
      }
    }, WAIT_TIMEOUT);
  };
  setTimeout(main, 2000);
})();
