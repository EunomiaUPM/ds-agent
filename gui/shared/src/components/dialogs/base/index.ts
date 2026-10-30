/**
 * Dialog Base Components Index
 *
 * Exports all base components and utilities for building process dialogs.
 */

export { BaseProcessDialog, type BaseProcessDialogProps, type ButtonVariant } from "@eunomia/ui";

export {
  // Contract Negotiation mappers
  mapCNProcessToInfoItems,
  mapCNProcessToInfoItemsForProvider,
  mapCNProcessToInfoItemsForConsumer,
  mapCNProcessToFullInfoItems,
  // Transfer Process mappers
  mapTransferProcessToInfoItems,
  mapTransferProcessToInfoItemsForProvider,
  mapTransferProcessToInfoItemsForConsumer,
  // Agreement mapper
  mapAgreementToInfoItems,
  // Generic helpers
  textInfoItem,
  urnInfoItem,
  statusInfoItem,
  dateInfoItem,
  roleInfoItem,
} from "./infoItemMappers";
