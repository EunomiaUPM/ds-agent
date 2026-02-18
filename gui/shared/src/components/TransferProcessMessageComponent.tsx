import React from "react";

import {
  MessageBody,
  MessageContent,
  MessageLog,
  MessageMeta,
  MessageMetaContainer,
  MessageTimestamp,
  MessageTitle,
  RoleHeader,
  RoleType,
} from "./ui/message";
import { TransferMessageDto } from "../data/orval/model";

type TransferProcessMessageComponentProps = {
  message: TransferMessageDto;
};
let addSpacesFormat = (text: string) => {
  return text.replace(/(?!^)([A-Z])/g, " $1");
};

/**
 * Component for displaying a Transfer Process message.
 */
const TransferProcessMessageComponent: React.FC<TransferProcessMessageComponentProps> = ({
  message,
}) => {
  return (
    <MessageLog key={message.id} variant={message.stateTransitionFrom as RoleType}>
      <RoleHeader from={message.stateTransitionFrom as RoleType} />
      <MessageBody variant={message.stateTransitionFrom as RoleType}>
        <MessageTitle className="text-brand-sky mb-0 pb-0">
          {addSpacesFormat(message.messageType || "")}
        </MessageTitle>
        <MessageTimestamp created_at={message.createdAt?.toString() || ""} />
        <MessageMetaContainer>
          <MessageMeta label=" Transfer Message Id" value={message.id?.slice(9, 60) || ""} />
          <MessageMeta
            label=" Transfer Process Id"
            value={message.transferAgentProcessId?.slice(9, 60) || ""}
          />
        </MessageMetaContainer>
        <MessageContent content={JSON.stringify(message.payload || {}, null, 4)} />
      </MessageBody>
    </MessageLog>
  );
};

export default TransferProcessMessageComponent;
