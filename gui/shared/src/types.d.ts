declare global {
  // export type UUID = string;
  // export type Urn = string;
  // export type DateTime = string; // ISO 8601 string

  // // --- CATALOG AGENT ---

  // export interface CatalogDto {
  //   id: Urn;
  //   foafHomePage?: string | null;
  //   dctConformsTo?: string | null;
  //   dctCreator?: string | null;
  //   dctIdentifier?: string | null;
  //   dctIssued: DateTime;
  //   dctModified?: DateTime | null;
  //   dctTitle?: string | null;
  //   dspaceParticipantId?: string | null;
  //   dspaceMainCatalog: boolean;
  // }

  // export interface DataServiceDto {
  //   id: Urn;
  //   dcatEndpointDescription?: string | null;
  //   dcatEndpointUrl: string;
  //   dctConformsTo?: string | null;
  //   dctCreator?: string | null;
  //   dctIdentifier?: string | null;
  //   dctIssued: DateTime;
  //   dctModified?: DateTime | null;
  //   dctTitle?: string | null;
  //   dctDescription?: string | null;
  //   catalogId: string;
  //   dspaceMainDataService: boolean;
  // }

  // export interface DatasetDto {
  //   id: Urn;
  //   dctConformsTo?: string | null;
  //   dctCreator?: string | null;
  //   dctIdentifier?: string | null;
  //   dctIssued: DateTime;
  //   dctModified?: DateTime | null;
  //   dctTitle?: string | null;
  //   dctDescription?: string | null;
  //   catalogId: string;
  // }

  // export interface DistributionDto {
  //   id: Urn;
  //   dctIssued: DateTime;
  //   dctModified?: DateTime | null;
  //   dctTitle?: string | null;
  //   dctDescription?: string | null;
  //   dcatAccessService: string;
  //   datasetId: string;
  //   dctFormat?: string | null;
  // }

  // export interface OdrlPolicyDto {
  //   id: Urn;
  //   odrlOffer: any;
  //   entity: string;
  //   entityType: string;
  //   createdAt: DateTime;
  //   sourceTemplateId?: string | null;
  //   sourceTemplateVersion?: string | null;
  //   instantiationParameters?: any | null;
  // }

  // export interface PolicyTemplateDto {
  //   id: string;
  //   version: string;
  //   date: DateTime;
  //   title?: LocalizedText | null;
  //   description?: LocalizedText | null;
  //   author: string;
  //   content: any; // OdrlPolicyInfo
  //   parameters: Record<string, ParameterDefinition>;
  // }

  // export type LocalizedText = Record<string, string> | string; // Simplified

  // export interface ParameterDefinition {
  //   type: string;
  //   label?: LocalizedText | null;
  //   description?: LocalizedText | null;
  //   default?: any;
  //   // Add other fields as necessary from policy_template.rs
  // }

  // // --- NEGOTIATION AGENT ---

  // export interface NegotiationProcessDto {
  //   id: Urn;
  //   state: string; // NegotiationProcessState
  //   stateAttribute?: string | null;
  //   associatedAgentPeer: string;
  //   protocol: string;
  //   callbackAddress?: string | null;
  //   role: string;
  //   properties: any;
  //   errorDetails?: any | null;
  //   createdAt: DateTime;
  //   updatedAt?: DateTime | null;

  //   // Virtual fields / Relations
  //   identifiers: Record<string, string>;
  //   messages: NegotiationMessageDto[];
  //   offers: NegotiationOfferDto[];
  //   agreement?: NegotiationAgreementDto | null;
  // }

  // export interface NegotiationMessageDto {
  //   id: Urn;
  //   negotiationAgentProcessId: string;
  //   createdAt: DateTime;
  //   direction: string;
  //   protocol: string;
  //   messageType: string;
  //   stateTransitionFrom: string;
  //   stateTransitionTo: string;
  //   payload: any;
  // }

  // export interface NegotiationOfferDto {
  //   id: Urn;
  //   negotiationAgentProcessId: string;
  //   negotiationAgentMessageId: string;
  //   offerId: string;
  //   offerContent: any;
  //   createdAt: DateTime;
  // }

  // export interface NegotiationAgreementDto {
  //   id: Urn;
  //   negotiationAgentProcessId: string;
  //   negotiationAgentMessageId: string;
  //   consumerParticipantId: string;
  //   providerParticipantId: string;
  //   agreementContent: any;
  //   target: string;
  //   state: string;
  //   createdAt: DateTime;
  //   updatedAt?: DateTime | null;
  // }

  // // --- TRANSFER AGENT ---

  // export interface TransferProcessDto {
  //   id: Urn;
  //   state: string; // TransferProcessState
  //   stateAttribute?: string | null;
  //   associatedAgentPeer: string;
  //   protocol: string;
  //   transferDirection: string;
  //   agreementId: string;
  //   callbackAddress?: string | null;
  //   role: string;
  //   properties: any;
  //   errorDetails?: any | null;
  //   createdAt: DateTime;
  //   updatedAt?: DateTime | null;

  //   // Virtual fields / Relations
  //   identifiers: Record<string, string>;
  //   messages: TransferMessageDto[];
  // }

  // export interface TransferMessageDto {
  //   id: Urn;
  //   transferAgentProcessId: string;
  //   createdAt: DateTime;
  //   direction: string;
  //   protocol: string;
  //   messageType: string;
  //   stateTransitionFrom: string;
  //   stateTransitionTo: string;
  //   payload?: any | null;
  // }

  // // --- PROTOCOL TYPES (DSP) ---

  // // Negotiation Protocol Types
  // export type NegotiationProcessState =
  //   | "REQUESTED"
  //   | "OFFERED"
  //   | "ACCEPTED"
  //   | "AGREED"
  //   | "VERIFIED"
  //   | "FINALIZED"
  //   | "TERMINATED";

  // export type NegotiationProcessMessageType =
  //   | "ContractRequestMessage"
  //   | "ContractOfferMessage"
  //   | "ContractNegotiationEventMessage"
  //   | "ContractAgreementMessage"
  //   | "ContractAgreementVerificationMessage"
  //   | "ContractNegotiationTerminationMessage"
  //   | "ContractNegotiation"
  //   | "ContractNegotiationError";

  // export interface ContractRequestMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  //   offer: any; // ContractRequestMessageOfferTypes
  //   callbackAddress?: string;
  // }

  // export interface ContractOfferMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  //   offer: any;
  //   callbackAddress?: string;
  // }

  // export interface ContractAgreementMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  //   agreement: any; // OdrlAgreement
  // }

  // export interface ContractAgreementVerificationMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  // }

  // export interface ContractNegotiationEventMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  //   eventType: "ACCEPTED" | "FINALIZED";
  // }

  // export interface ContractNegotiationTerminationMessage {
  //   consumerPid: Urn;
  //   providerPid: Urn;
  //   code?: string;
  //   reason?: string[];
  // }

  // // Transfer Protocol Types
  // export type TransferProcessState =
  //   | "REQUESTED"
  //   | "STARTED"
  //   | "COMPLETED"
  //   | "SUSPENDED"
  //   | "TERMINATED";

  // export type TransferProcessMessageType =
  //   | "TransferRequestMessage"
  //   | "TransferStartMessage"
  //   | "TransferCompletionMessage"
  //   | "TransferSuspensionMessage"
  //   | "TransferTerminationMessage"
  //   | "TransferProcess"
  //   | "TransferError";

  // export interface TransferRequestMessage {
  //   agreementId: Urn;
  //   format: string;
  //   dataAddress?: any; // DataAddressDto
  //   callbackAddress: string;
  //   consumerPid: Urn;
  // }

  // export interface TransferStartMessage {
  //   providerPid: Urn;
  //   consumerPid: Urn;
  //   dataAddress?: any;
  // }

  // export interface TransferCompletionMessage {
  //   providerPid: Urn;
  //   consumerPid: Urn;
  // }

  // export interface TransferTerminationMessage {
  //   providerPid: Urn;
  //   consumerPid: Urn;
  //   code?: string;
  //   reason?: string[];
  // }

  // export interface TransferSuspensionMessage {
  //   providerPid: Urn;
  //   consumerPid: Urn;
  //   code?: string;
  //   reason?: string[];
  // }

  // // --- MISC ---

  // export interface ConnectorMetadata {
  //   name?: string;
  //   author?: string;
  //   description?: string;
  //   version?: string;
  //   createdAt?: DateTime;
  // }

  // export interface ConnectorInstanceDto extends ConnectorMetadata {
  //   id: string;
  //   authenticationConfig: any; // Simplified
  //   interaction: any; // Simplified
  //   distributionId: string;
  // }

  // export interface ParticipantDto {
  //   participantId: string;
  //   participantSlug: string;
  //   participantType: string;
  //   baseUrl: string;
  //   token: any;
  //   tokenActions: any;
  //   savedAt: DateTime;
  //   lastInteraction: DateTime;
  //   isMe: boolean;
  // }
}

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export {};
