import React from "react";
import { formatUrn } from "shared/src/lib/utils";
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
import { NegotiationMessageDto } from "../data/orval/model";

type MessageComponentProps = {
  message: NegotiationMessageDto;
};
let addSpacesFormat = (text: string) => {
  if (!text) return "";
  return text.replace(/(?!^)([A-Z])/g, " $1");
};

/**
 * Component for displaying a Contract Negotiation process message.
 */
const MessageComponent: React.FC<MessageComponentProps> = ({ message }) => {
  return (
    <MessageLog key={message.id} variant={message.stateTransitionFrom as RoleType}>
      <RoleHeader from={message.stateTransitionFrom as RoleType} />
      <MessageBody variant={message.stateTransitionFrom as RoleType}>
        <MessageTitle className="text-brand-sky mb-0 pb-0">
          {addSpacesFormat(message.messageType)}
        </MessageTitle>
        <MessageTimestamp created_at={message.createdAt.toString()} />
        <MessageMetaContainer>
          <MessageMeta label="Contract Message Id" value={formatUrn(message.id, false)} />
          <MessageMeta
            label="Contract Process Id"
            value={formatUrn(message.negotiationAgentProcessId, false)}
          />
        </MessageMetaContainer>
        <MessageContent content={JSON.stringify(message.payload, null, 8)} />
      </MessageBody>
    </MessageLog>
  );
};

export default MessageComponent;
