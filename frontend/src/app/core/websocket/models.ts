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
}
