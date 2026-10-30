// src/json/MonacoJson.tsx
import { useState } from "react";
import Editor, { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor/esm/vs/editor/edcore.main";
import "monaco-editor/esm/vs/language/json/monaco.contribution";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import JsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import { jsx } from "react/jsx-runtime";
self.MonacoEnvironment = {
  getWorker: (_id, label) => label === "json" ? new JsonWorker() : new EditorWorker()
};
loader.config({ monaco });
var LINE = 18;
var PADDING = 12;
function MonacoJson({ text, dark, maxHeight }) {
  const [height, setHeight] = useState(
    () => Math.min(text.split("\n").length * LINE + PADDING, maxHeight)
  );
  const onMount = (editor) => {
    const fit = () => setHeight(Math.min(editor.getContentHeight(), maxHeight));
    editor.onDidContentSizeChange(fit);
    fit();
  };
  return /* @__PURE__ */ jsx(
    Editor,
    {
      height,
      language: "json",
      value: text,
      theme: dark ? "vs-dark" : "vs",
      onMount,
      loading: null,
      options: {
        readOnly: true,
        domReadOnly: true,
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        automaticLayout: true,
        folding: true,
        showFoldingControls: "mouseover",
        lineNumbers: "on",
        lineNumbersMinChars: 3,
        glyphMargin: false,
        wordWrap: "on",
        fontSize: 12,
        lineHeight: LINE,
        fontFamily: 'Menlo, Monaco, Consolas, "Courier New", monospace',
        fontLigatures: false,
        renderLineHighlight: "none",
        overviewRulerLanes: 0,
        hideCursorInOverviewRuler: true,
        stickyScroll: { enabled: false },
        guides: { indentation: true, bracketPairs: true },
        bracketPairColorization: { enabled: true },
        contextmenu: false,
        padding: { top: 6, bottom: 6 },
        scrollbar: {
          alwaysConsumeMouseWheel: false,
          verticalScrollbarSize: 8,
          horizontalScrollbarSize: 8
        }
      }
    }
  );
}
export {
  MonacoJson as default
};
//# sourceMappingURL=MonacoJson-WVP24WDG.js.map