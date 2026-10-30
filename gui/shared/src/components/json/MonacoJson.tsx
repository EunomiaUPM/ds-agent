// Monaco set up from the bundle, never a CDN: the editor core with its features (find, folding)
// and only the JSON language, with their workers. Loaded lazily by JsonViewer.

import React, { useState } from "react";
import Editor, { loader, type OnMount } from "@monaco-editor/react";
import * as monaco from "monaco-editor/esm/vs/editor/edcore.main";
import "monaco-editor/esm/vs/language/json/monaco.contribution";
import EditorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import JsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";

self.MonacoEnvironment = {
  getWorker: (_id: string, label: string) =>
    label === "json" ? new JsonWorker() : new EditorWorker(),
};
loader.config({ monaco });

// VS Code's defaults on macOS: Menlo 12px, 18px lines.
const LINE = 18;
const PADDING = 12;

export interface MonacoJsonProps {
  text: string;
  dark: boolean;
  /** Tallest the editor grows, in pixels; it is as tall as its content up to there. */
  maxHeight: number;
}

/** Read-only JSON in Monaco: line numbers, folding, find (Ctrl/Cmd+F), bracket guides. */
export default function MonacoJson({ text, dark, maxHeight }: MonacoJsonProps) {
  const [height, setHeight] = useState(() =>
    Math.min(text.split("\n").length * LINE + PADDING, maxHeight),
  );
  const onMount: OnMount = (editor) => {
    const fit = () => setHeight(Math.min(editor.getContentHeight(), maxHeight));
    editor.onDidContentSizeChange(fit);
    fit();
  };
  return (
    <Editor
      height={height}
      language="json"
      value={text}
      theme={dark ? "vs-dark" : "vs"}
      onMount={onMount}
      loading={null}
      options={{
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
          horizontalScrollbarSize: 8,
        },
      }}
    />
  );
}
