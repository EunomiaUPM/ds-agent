/** Maps dataplane state values (both Pascal and UPPER) to badge state keys */
export function dataplaneStateVariant(state?: string | null): string {
  switch (state?.toLowerCase()) {
    case "init":
      return "REQUESTED";
    case "configuring":
      return "ACTIVE";
    case "auth":
      return "ACCEPTED";
    case "ready":
      return "AGREED";
    case "subscribing":
      return "STARTED";
    case "started":
      return "STARTED";
    case "unsubscribing":
      return "SUSPENDED";
    case "stopped":
      return "STOPPED";
    case "terminated":
      return "TERMINATED";
    case "error":
      return "TERMINATED";
    default:
      return "default";
  }
}

/** Maps TransferRole to display label */
export function roleLabel(role?: string): "Provider" | "Consumer" {
  return role?.toUpperCase() === "CONSUMER" ? "Consumer" : "Provider";
}
