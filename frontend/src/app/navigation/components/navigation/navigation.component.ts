import {Component, OnInit} from '@angular/core';
import {MainService} from '../../../core/services/main.service';
import {Alert} from '../../../core/models';
import {WebSocketService} from '../../../core/services/web-socket.service';
import {ConnectionStatus} from '../../../core/websocket/models';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrls: ['./navigation.component.scss']
})
export class NavigationComponent implements OnInit {

  alerts: Alert[] = [];

  connectionStatus: ConnectionStatus;
  statuses = ConnectionStatus;
  willReconnectIn = 0;

  constructor(
    private service: MainService,
    private webSocketService: WebSocketService
  ) {
    this.service.listenAlerts().subscribe(alert => this.alerts.push(alert));

    this.webSocketService.listenConnectionStatus().subscribe(status => this.connectionStatus = status);

    this.webSocketService.willReconnectIn().subscribe(time => this.willReconnectIn = time);

    this.webSocketService.listenNotifications().subscribe(notification => {
      this.service.alertSuccess(JSON.stringify(notification.message));
    });
  }

  ngOnInit(): void {
  }

  toggleNavbar(navbarId: string): void {
    const target = document.getElementById(navbarId);

    if (target) {
      target.classList.toggle('show');
    }
  }

  removeAlert(alert: Alert): void {
    const index = this.alerts.findIndex(a => a === alert);

    if (index !== -1) {
      this.alerts.splice(index, 1);
    }
  }

  reconnect(): void {
    this.webSocketService.connect();
  }
}
