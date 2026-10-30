// Assets
export * from "./assets";

// Navigation / Router Link Abstraction
export * from "./link";

// Theme & Hooks
export * from "./theme/ThemeContext";
export * from "./hooks/useTheme";
export * from "./hooks/use-mobile";
export * from "./hooks/useTableQueryParams";

// Lib (Utils, HTTP / Orval Mutator, Session)
export * from "./lib/utils";
export * from "./lib/http";
export {
  type SessionUser,
  getSessionUser,
  sessionDisplayName,
  loadSession,
  signIn,
  signOut,
} from "./lib/session";

// UI Primitives & Formatters
export * from "./ui/accordion";
export { default as Avatar, default as AvatarImg } from "./ui/avatar-img";
export * from "./ui/badge";
export { BooleanBadge, type BooleanBadgeProps } from "./ui/boolean-badge";
export * from "./ui/breadcrumb";
export * from "./ui/bubble-background";
export * from "./ui/button";
export * from "./ui/card";
export * from "./ui/chart";
export * from "./ui/checkbox";
export * from "./ui/command";
export * from "./ui/confirm-dialog";
export * from "./ui/count-badge";
export * from "./ui/dialog";
export * from "./ui/drawer";
export * from "./ui/dropdown-menu";
export * from "./ui/form";
export * from "./ui/format-date";
export * from "./ui/format-key";
export * from "./ui/format-operator";
export * from "./ui/format-value";
export { default as Heading, type HeadingLevel, type HeadingProps } from "./ui/heading";
export * from "./ui/info-list";
export * from "./ui/input";
export * from "./ui/label";
export * from "./ui/list";
export * from "./ui/message";
export { default as NoFurtherActions } from "./ui/noFurtherActions";
export * from "./ui/popover";
export * from "./ui/segmented";
export * from "./ui/select";
export * from "./ui/separator";
export * from "./ui/sheet";
export * from "./ui/sidebar";
export * from "./ui/skeleton";
export * from "./ui/sonner";
export * from "./ui/table";
export * from "./ui/tabs";
export * from "./ui/tag-input";
export * from "./ui/textarea";
export * from "./ui/theme-toggle";
export * from "./ui/tooltip";

// Layout & Dashboard
export * from "./layout/FieldGrid";
export * from "./layout/InfoGrid";
export * from "./layout/InnerSidebarLayout";
export * from "./layout/PageHeader";
export * from "./layout/PageLayout";
export * from "./layout/PageSection";
export * from "./layout/RouteTabs";
export * from "./dashboard/FeatureCard";
export * from "./dashboard/StatTile";

// Table
export * from "./table/DataTable";
export * from "./table/SortableHeader";

// Identity & Process
export * from "./identity/Identifier";
export * from "./identity/ParticipantLabel";
export * from "./process/StateStepper";
export { default as WizardDialog } from "./process/WizardDialog";
export { default as WizardEndDialog, type WizardEndDialogProps } from "./process/WizardEndDialog";
export {
  BaseProcessDialog,
  type BaseProcessDialogProps,
  type ButtonVariant,
} from "./process/BaseProcessDialog";

// Shell
export * from "./shell/AppShellSidebar";
export * from "./shell/AppShellHeader";
export * from "./shell/SignInScreen";
export * from "./shell/GeneralErrorComponent";
export * from "./shell/GlobalLoadingIndicator";

// JSON Viewer & Table
export * from "./json";

// SSI Wallet (6 views + layout)
export * from "./wallet";
