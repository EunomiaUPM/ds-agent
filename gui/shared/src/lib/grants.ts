// Badge states of grants and verifications with GNAP rounds; to move to @eunomia/ui.

/**
 * Badge state of a grant status (RFC 9635 §1.5: no rejected grant, a denied one is finalized).
 * A finalized grant is red unless `succeeded`, as a credential request whose credential was
 * claimed.
 */
export const grantBadgeState = (
  status: string | undefined,
  succeeded = false,
): string | undefined => {
  switch (status) {
    case "Processing":
      return "running";
    case "Pending":
      return "pending";
    case "Approved":
      return "success";
    case "Finalized":
      return succeeded ? "success" : "failed";
    default:
      return status;
  }
};

/** Badge state of a verification status: anything but pending or verified is red. */
export const verificationBadgeState = (status: string | undefined): string | undefined => {
  switch (status) {
    case "Pending":
      return "pending";
    case "Verified":
      return "success";
    case "Failed":
    case "Refused":
    case "Expired":
      return "failed";
    default:
      return status;
  }
};
