import {JobStatus} from '../../experiment/models';

export enum ConnectionStatus {
  Initial,
  Open,
  Closed,
  Reconnecting,
}

export enum OutgoingMessageKind {
  KeepAlive = 'KeepAlive',
}

export enum IncomingMessageKind {
  Error = 'Error',
  Notification = 'Notification',
}

export enum NotificationKind {
  JobUpdate = 'JobUpdate'
}

export class NotificationData {
}

export interface Notification<T> {
  userId: number;
  message: NotificationMessage<T>;
}

export interface NotificationMessage<T extends NotificationData> {
  kind: NotificationKind;
  data: T;
}
