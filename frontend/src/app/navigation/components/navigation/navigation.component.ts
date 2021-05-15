import {Component, OnInit} from '@angular/core';
import {MainService} from '../../../core/services/main.service';
import {Alert} from '../../../core/models';
import {WebSocketService} from '../../../core/services/web-socket.service';
import {ConnectionStatus, Notification, NotificationKind} from '../../../core/websocket/models';
import {JobUpdate} from '../../../experiment/models';
import {UserViewModelService} from '../../../user/services/user-view-model.service';
import {Role, User} from '../../../user/models';
import {MainComponent} from '../../../shared/components/main/main.component';
import {ActivatedRoute} from '@angular/router';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrls: ['./navigation.component.scss']
})
export class NavigationComponent extends MainComponent implements OnInit {

  menus = [];

  alerts: Alert[] = [];

  connectionStatus: ConnectionStatus;

  statuses = ConnectionStatus;
  willReconnectIn = 0;

  constructor(
    private service: MainService,
    private userViewModel: UserViewModelService,
    private webSocketService: WebSocketService,
    private route: ActivatedRoute,
  ) {
    super();

    this.service.listenAlerts().subscribe(alert => {
      this.alerts.push(alert);
      setTimeout(() => this.removeAlert(alert), alert.timeout ?? 2500);
    });

    this.subs.add(this.webSocketService.listenConnectionStatus().subscribe(status => this.connectionStatus = status));

    this.subs.add(this.webSocketService.willReconnectIn().subscribe(time => this.willReconnectIn = time));

    this.subs.add(this.webSocketService.listenNotifications().subscribe(notification => {
      this.service.alertSuccess(NavigationComponent.getNotificationMessage(notification));
    }));
  }

  private static getNotificationMessage(notification: Notification<any>): string {
    if (notification.message.kind === NotificationKind.JobUpdate) {
      const data = notification.message.data as JobUpdate;
      return `Job ${data.jobId} is updated with status ${data.status}`;
    }
  }

  ngOnInit(): void {
    this.subs.add(
      this.route.data.subscribe((data: { user: User }) => {
        if (data.user.roleId === Role.Admin) {
          this.menus = [profile, experiments, runners, settings, logout];
        } else {
          this.menus = [profile, experiments, settings, logout];
        }
      })
    );
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

const profile = {
  routerLink: '/user/profile',
  text: 'Profile',
};
const settings = {
  routerLink: '/user/settings',
  text: 'Settings',
};

const experiments = {
  routerLink: '/experiment/experiments',
  text: 'Experiments',
};

const runners = {
  routerLink: '/admin/runners',
  text: 'Runners',
};

const logout = {
  routerLink: '/auth/logout',
  text: 'Logout',
};
