
import { useContext } from "react";
import { Circle } from "lucide-react";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { PeerConnectorForm } from "../forms/PeerConnectorForm";
import { GrantRequestForm } from "../forms/GrantRequestForm";
import { VPPresentationForm } from "../forms/VPPresentationForm";

export function PeerStep() {
  const ssiAuthContext = useContext(SSIAuthContext);

  return (
    <div className="space-y-6 mt-4">
      <div className="space-y-4">
        <div>
          <h3 className="text-lg font-medium">4. Connect to Peer</h3>
          <p className="text-muted-foreground text-sm">
            Fetch the Peer DID to start the authentication process.
          </p>
        </div>
        <div className="w-full border rounded-lg p-4 space-y-4">
          <PeerConnectorForm />
          <div>
            {ssiAuthContext.tempPeer.did ? (
              <div className="text-sm font-mono p-3 bg-muted rounded break-all border">
                <span className="font-semibold select-none mr-2">Peer DID:</span>
                {ssiAuthContext.tempPeer.did}
              </div>
            ) : (
              <div className="text-sm text-muted-foreground italic">
                Fetch peer DID to proceed.
              </div>
            )}
          </div>
        </div>
      </div>

      <hr />

      <div className="space-y-4">
        <h3 className="text-lg font-medium">5. Authenticate</h3>

        <div className="border rounded-lg p-4 space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">
              Step A: Request Authentication Grant
            </span>
            <GrantRequestForm />
          </div>

          {ssiAuthContext.oidc4vpRequestUri && (
            <div className="space-y-4 pt-4 border-t">
              <div className="text-sm font-mono p-3 bg-muted rounded break-all border">
                <span className="font-semibold select-none block mb-1">
                  Presentation Request URI:
                </span>
                {ssiAuthContext.oidc4vpRequestUri}
              </div>

              <div className="flex items-center justify-between bg-secondary/20 p-4 rounded-lg">
                <span className="text-sm font-medium">
                  Step B: Present Verified Presentation
                </span>
                <VPPresentationForm />
              </div>
            </div>
          )}

          {ssiAuthContext.oidc4vpSuccess && (
            <div className="p-4 bg-green-50 border border-green-200 rounded-lg text-green-800 flex items-center gap-3">
              <Circle className="h-5 w-5 fill-current" />
              <div>
                <h4 className="font-semibold">Authentication Successful!</h4>
                <p className="text-sm">
                  You can now proceed to Participant List, fetch the Peer Catalog,
                  and start transfers.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
