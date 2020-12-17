import {Injectable} from '@angular/core';
import {Subject} from 'rxjs';
import {Alert} from '../models';

@Injectable({
  providedIn: 'root'
})
export class MainService {

  private alerts = new Subject<Alert>();

  constructor() {
  }

  alertSuccess(text: string): void {
    this.alert({
      text,
      type: 'success',
      icon: 'check-circle'
    });
  }

  alertWarning(text: string): void {
    this.alert({
      text,
      type: 'warning',
      icon: 'minus-circle'
    });
  }

  alertFail(text: string): void {
    this.alert({
      text,
      type: 'danger',
      icon: 'times-circle'
    });
  }

  alert(alert: Alert): void {
    this.alerts.next(alert);
  }

  listenAlerts(): Subject<Alert> {
    return this.alerts;
  }
}
