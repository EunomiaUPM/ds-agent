
import { useContext } from "react";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { WalletOnboardForm } from "../forms/WalletOnboardForm";
import Heading from "../../../../../shared/src/components/ui/heading.tsx";

export function WalletStep() {
  const ssiAuthContext = useContext(SSIAuthContext);

  return (
    <div className="space-y-4 mt-4">
      <div className="flex flex-col gap-2 mb-2">
        <h3 className="text-lg font-medium">1. Wallet Onboarding</h3>
        <p className="text-muted-foreground text-sm">
          First, check if you are onboarded in a wallet and view your DID.
        </p>
      </div>
      <div className="w-full border rounded-lg p-4">
        <WalletOnboardForm />
        <div className="mt-4">
          {ssiAuthContext.ownDid ? (
            <div className="text-sm font-mono p-3 bg-muted rounded break-all border">
              <span className="font-semibold select-none mr-2">Your DID:</span>
              {ssiAuthContext.ownDid}
            </div>
          ) : (
            <div className="text-sm text-muted-foreground italic">
              No DID found. Please onboard first.
            </div>
          )}
        </div>
      </div>
      
    </div>
  );
}
