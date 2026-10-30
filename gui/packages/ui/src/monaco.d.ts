declare module "monaco-editor/esm/vs/editor/edcore.main" {
  const monaco: typeof import("monaco-editor");
  export = monaco;
}

declare module "monaco-editor/esm/vs/language/json/monaco.contribution";

declare module "*?worker" {
  const WorkerConstructor: {
    new (): Worker;
  };
  export default WorkerConstructor;
}

declare module "*.svg" {
  const content: string;
  export default content;
}

declare module "*.png" {
  const content: string;
  export default content;
}
