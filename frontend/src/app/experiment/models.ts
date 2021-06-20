import {NotificationData} from '../core/websocket/models';

export interface Experiment {
  id: number;
  userId: number;
  name: string;
  code: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface SlimRunner {
  id: number;
  name: string;
  createdAt: Date;
}

export interface SlimJob {
  id: number;
  experimentId: number;
  runnerId: number;
  status: JobStatus;
  createdAt: Date;
  updatedAt: Date;
}

export interface Job extends SlimJob {
  code: string;
}


export interface JobUpdate extends NotificationData {
  jobId: number;
  status: JobStatus;
}

export enum JobStatus {
  Pending = 'Pending',
  Running = 'Running',
  Successful = 'Successful',
  Failed = 'Failed'
}
