import { useRouter } from "@tanstack/react-router";
import { AlertCircle, RotateCcw } from "lucide-react";
import { Button } from "shared/src/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "shared/src/components/ui/card";

interface GeneralErrorComponentProps {
  error: Error;
  reset: () => void;
}

/**
 * Component for displaying a general error message.
 */
export function GeneralErrorComponent({ error, reset }: GeneralErrorComponentProps) {
  const router = useRouter();

  return (
    <div className="flex h-[50vh] w-full items-center justify-center p-4">
      <Card className="w-full max-w-md border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950/20">
        <CardHeader>
          <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
            <AlertCircle className="h-5 w-5" />
            <CardTitle>Something went wrong!</CardTitle>
          </div>
          <CardDescription className="text-red-600/90 dark:text-red-400/90">
            An error occurred while loading this page.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="rounded-md bg-white/50 p-3 text-sm font-medium text-red-800 dark:bg-sunken/20 dark:text-red-200">
            {error.message || "Unknown error"}
          </div>
        </CardContent>
        <CardFooter className="flex justify-end gap-2">
          <Button
            variant="outline"
            className="border-red-200 text-red-700 hover:bg-red-100 dark:border-red-800 dark:text-red-300 dark:hover:bg-red-900/50"
            onClick={() => window.history.back()}
          >
            Go Back
          </Button>
          <Button
            variant="default"
            className="bg-red-600 text-white hover:bg-red-700 dark:bg-red-700 dark:hover:bg-red-600"
            onClick={() => {
              // Invalidate the router to retry loaders
              router.invalidate();
              // Reset the error boundary
              reset();
            }}
          >
            <RotateCcw className="mr-2 h-4 w-4" />
            Try Again
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
