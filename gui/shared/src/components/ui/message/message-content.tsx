import * as React from "react";
import { cn } from "shared/src/lib/utils";
import SyntaxHighlighter from "react-syntax-highlighter";
import { vs2015 } from "react-syntax-highlighter/dist/esm/styles/hljs";

export const MessageContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => {
  const content = (props as any).content;
  return (
    <div ref={ref} className={cn("flex flex-col gap-3", className)} {...props}>
      <p className="font-bold min-w-40 text-white/60">Content:</p>
      <div className="w-full break-all">
        <pre className="p-4 rounded-lg break-all text-[13px] !font-mono overflow-hidden bg-black/70 text-secondary-300">
          <code className="whitespace-pre break-all">
            <SyntaxHighlighter
              style={vs2015}
              language="json"
              wrapLongLines={false}
              showLineNumbers={true}
            >
              {content}
            </SyntaxHighlighter>
          </code>
        </pre>
      </div>
    </div>
  );
});
MessageContent.displayName = "MessageContent";
