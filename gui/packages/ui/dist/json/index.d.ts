import * as react_jsx_runtime from 'react/jsx-runtime';

interface JsonViewerProps {
    value: unknown;
    className?: string;
    /** Tailwind max-height class for the editor, such as `max-h-[60vh]` or `max-h-96`. */
    maxHeightClass?: string;
}
/** Pretty-printed JSON in a read-only Monaco editor framed like a VS Code tab, with a copy button. */
declare function JsonViewer({ value, className, maxHeightClass }: JsonViewerProps): react_jsx_runtime.JSX.Element;
declare namespace JsonViewer {
    var prettify: (raw: string) => string;
    var pixels: (heightClass: string) => number;
}

type Json = null | boolean | number | string | Json[] | {
    [key: string]: Json;
};
interface JsonTableProps {
    value: unknown;
    className?: string;
    /** Nesting levels expanded on first render. */
    expandDepth?: number;
    /** Shows a Table/JSON switch above the table. */
    rawToggle?: boolean;
    /** Text shown when the value is empty. */
    emptyText?: string;
}
/** Any JSON value as a nested key/value table, with an optional switch to the raw JSON. */
declare function JsonTable({ value, className, expandDepth, rawToggle, emptyText, }: JsonTableProps): react_jsx_runtime.JSX.Element;
declare namespace JsonTable {
    var normalize: (value: unknown) => Json;
    var isEmpty: (v: Json) => boolean;
    var isComposite: (v: Json) => v is Json[] | {
        [key: string]: Json;
    };
    var summary: (v: Json[] | {
        [key: string]: Json;
    }) => string;
    var Node: ({ value, depth, expandDepth, }: {
        value: Json;
        depth: number;
        expandDepth: number;
    }) => react_jsx_runtime.JSX.Element;
    var Row: ({ name, value, depth, expandDepth, }: {
        name: string;
        value: Json;
        depth: number;
        expandDepth: number;
    }) => react_jsx_runtime.JSX.Element;
    var Primitive: ({ value }: {
        value: Json;
    }) => react_jsx_runtime.JSX.Element;
}

interface MonacoJsonProps {
    text: string;
    dark: boolean;
    /** Tallest the editor grows, in pixels; it is as tall as its content up to there. */
    maxHeight: number;
}

export { JsonTable, type JsonTableProps, JsonViewer, type JsonViewerProps, type MonacoJsonProps };
