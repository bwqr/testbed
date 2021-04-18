export interface User {
  id: number;
  firstName: string;
  lastName: string;
  email: string;
  status: UserStatus;
  roleId: Role;
}

export enum UserStatus {
  NotVerified = 'NotVerified',
  Verified = 'Verified',
  Banned = 'Banned',
}

export enum Role {
  Admin = 1,
  User
}
