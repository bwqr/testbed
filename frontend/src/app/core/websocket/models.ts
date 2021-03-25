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

export interface Notification<T> {
  userId: number;
  message: NotificationMessage<T>;
}

export interface NotificationMessage<T> {
  kind: NotificationKind;
  data: T;
}

export enum NotificationKind {
  JobUpdate = 'JobUpdate'
}
